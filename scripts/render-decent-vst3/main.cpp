// Headless diagnostic host using Steinberg's public VST3 SDK directly.
// No JUCE/Pedalboard MIDI conversion, plug-in editor, or audio device.
#include "public.sdk/source/vst/hosting/module.h"
#include "public.sdk/source/vst/hosting/plugprovider.h"
#include "public.sdk/source/vst/hosting/hostclasses.h"
#include "public.sdk/source/vst/hosting/processdata.h"
#include "public.sdk/source/vst/hosting/eventlist.h"
#include "public.sdk/source/common/memorystream.h"
#include "pluginterfaces/vst/ivstaudioprocessor.h"
#include <CoreFoundation/CoreFoundation.h>
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <cerrno>
#include <cstring>
#include <memory>
#include <streambuf>
#include <filesystem>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <vector>

using namespace Steinberg;
using namespace Steinberg::Vst;
constexpr int rate = 48000, block = 256;
struct Message { int64_t frame; int note, velocity; bool on; };

void require(bool ok, const char* description) { if (!ok) throw std::runtime_error(description); }
// Reserve output names atomically, then write through the same descriptors.
// This handles case-insensitive aliases and avoids reopening a replaced path.
class ExclusiveOutput : public std::streambuf {
    int fd = -1;
    std::filesystem::path path;
    bool keep = false;
    char buffer[65536];
    std::streamsize writeAll(const char* bytes, std::streamsize count) {
        std::streamsize done = 0;
        while (done < count) {
            const auto written = ::write(fd, bytes + done, count - done);
            if (written < 0 && errno == EINTR) continue;
            if (written <= 0) break;
            done += written;
        }
        return done;
    }
    int sync() override {
        const auto count = pptr() - pbase();
        if (writeAll(buffer, count) != count) return -1;
        setp(buffer, buffer + sizeof(buffer));
        return 0;
    }
    std::streamsize xsputn(const char* bytes, std::streamsize count) override {
        std::streamsize done = 0;
        while (done < count) {
            if (pptr() == epptr() && sync() != 0) break;
            const auto available = static_cast<std::streamsize>(epptr() - pptr());
            const auto chunk = std::min(available, count - done);
            std::memcpy(pptr(), bytes + done, chunk);
            pbump(static_cast<int>(chunk)); done += chunk;
        }
        return done;
    }
    int_type overflow(int_type character) override {
        if (traits_type::eq_int_type(character, traits_type::eof())) return traits_type::not_eof(character);
        const char byte = traits_type::to_char_type(character);
        return xsputn(&byte, 1) == 1 ? character : traits_type::eof();
    }
public:
    explicit ExclusiveOutput(const char* name) : path(name) {
        setp(buffer, buffer + sizeof(buffer));
        fd = ::open(name, O_WRONLY | O_CREAT | O_EXCL, 0600);
        if (fd < 0) throw std::runtime_error("Cannot exclusively create output: " + path.string() + ": " + std::strerror(errno));
    }
    ~ExclusiveOutput() {
        if (fd >= 0) {
            // Delete a failed reservation only if this pathname still names our inode.
            struct stat owned {}, current {};
            if (!keep && ::fstat(fd, &owned) == 0 && ::lstat(path.c_str(), &current) == 0 &&
                owned.st_dev == current.st_dev && owned.st_ino == current.st_ino)
                ::unlink(path.c_str());
            ::close(fd);
        }
    }
    void finish() {
        require(sync() == 0, "Failed to write output buffer");
        require(::fsync(fd) == 0, "Failed to flush output");
    }
    void commit() { keep = true; }
};
void waitForPlayer() {
    for (int i = 0; i < 300; ++i)
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.01, false);
}
void little(std::ostream& stream, uint32_t value, int bytes = 4) {
    for (int i = 0; i < bytes; ++i) stream.put(static_cast<char>(value >> (8 * i)));
}

int main(int argc, char** argv) try {
    require(argc == 4 || argc == 6 || argc == 7,
            "Usage: HOST PLUGIN --seed OUT.bin | HOST PLUGIN STATE.bin PROBES.csv OUT.wav EVENTS.csv [--nextafter]");
    require(std::getenv("CFFIXED_USER_HOME"), "Set an isolated CFFIXED_USER_HOME before loading the player");
    const bool seedMode = std::string(argv[2]) == "--seed";
    if (seedMode) {
        require(argc == 4, "--seed mode requires exactly plugin, --seed, and state output");
        require(!std::filesystem::exists(argv[3]), "Refusing to overwrite state output");
    } else {
        require(argc == 6 || argc == 7, "Rendering requires state, probes, output, event trace");
        require(argc != 7 || std::string(argv[6]) == "--nextafter", "Unknown diagnostic option");
        require(!std::filesystem::exists(argv[4]) && !std::filesystem::exists(argv[5]),
                "Refusing to overwrite render or event trace");
        require(std::filesystem::weakly_canonical(argv[4]) != std::filesystem::weakly_canonical(argv[5]),
                "Render and event trace outputs must have distinct paths");
    }
    std::vector<Message> messages;
    double endSeconds = 0;
    if (!seedMode) {
        std::ifstream probes(argv[3]);
        require(probes.good(), "Cannot read probes CSV");
        std::string line;
        std::getline(probes, line);
        while (std::getline(probes, line)) {
            std::istringstream row(line);
            std::vector<std::string> fields;
            std::string field;
            while (std::getline(row, field, ',')) fields.push_back(field);
            require(fields.size() == 6, "Unexpected probe format");
            std::size_t startEnd, noteEnd, velocityEnd;
            const double start = std::stod(fields[1], &startEnd);
            int note = std::stoi(fields[2], &noteEnd), velocity = std::stoi(fields[3], &velocityEnd);
            require(startEnd == fields[1].size() && noteEnd == fields[2].size() &&
                    velocityEnd == fields[3].size(), "Invalid MIDI probe number format");
            require(std::isfinite(start) && start >= 0 && start <= 3600 &&
                    note >= 0 && note <= 127 && velocity >= 1 && velocity <= 127, "Invalid MIDI probe values");
            messages.push_back({static_cast<int64_t>(std::llround(start * rate)), note, velocity, true});
            messages.push_back({static_cast<int64_t>(std::llround((start + .5) * rate)), note, 0, false});
            endSeconds = std::max(endSeconds, start + 2.5);
        }
        require(!messages.empty(), "No probes");
        std::stable_sort(messages.begin(), messages.end(), [](auto a, auto b) { return a.frame < b.frame; });
    }
    const int64_t frames = static_cast<int64_t>(endSeconds * rate);
    ExclusiveOutput outputFile(seedMode ? argv[3] : argv[4]);
    std::unique_ptr<ExclusiveOutput> traceFile;
    if (!seedMode) traceFile = std::make_unique<ExclusiveOutput>(argv[5]);
    std::string error;
    auto module = VST3::Hosting::Module::create(argv[1], error);
    require(static_cast<bool>(module), error.c_str());
    auto host = owned(new HostApplication());
    PluginContextFactory::instance().setPluginContext(host);
    module->getFactory().setHostContext(host);
    auto classes = module->getFactory().classInfos();
    auto info = std::find_if(classes.begin(), classes.end(), [](const auto& item) {
        return item.category() == kVstAudioEffectClass;
    });
    require(info != classes.end(), "No audio component in plugin");
    PlugProvider provider(module->getFactory(), *info, true);
    require(provider.initialize(), "Plugin initialization failed");
    auto component = provider.getComponentPtr();
    FUnknownPtr<IAudioProcessor> processor(component);
    require(static_cast<bool>(processor), "No audio processor interface");
    if (seedMode) {
        MemoryStream state;
        require(component->getState(&state) == kResultOk, "getState failed");
        std::ostream output(&outputFile);
        output.write(state.getData(), state.getSize());
        require(output.good(), "Failed to save component state");
        outputFile.finish(); outputFile.commit();
        std::cout << "Saved native component state, " << state.getSize() << " bytes\n";
        return 0;
    }
    require(argc >= 6, "Rendering requires state, probes, output, event trace");
    const bool roundUp = argc == 7 && std::string(argv[6]) == "--nextafter";
    std::ifstream stateFile(argv[2], std::ios::binary);
    std::vector<char> stateBytes((std::istreambuf_iterator<char>(stateFile)), {});
    require(!stateBytes.empty(), "State file empty or unreadable");
    MemoryStream state(stateBytes.data(), stateBytes.size());
    require(component->setState(&state) == kResultOk, "setState failed");
    auto controller = provider.getControllerPtr();
    if (controller) { state.seek(0, IBStream::kIBSeekSet, nullptr); controller->setComponentState(&state); }
    waitForPlayer();
    std::vector<SpeakerArrangement> inputs(component->getBusCount(kAudio, kInput));
    std::vector<SpeakerArrangement> outputs(component->getBusCount(kAudio, kOutput));
    for (std::size_t bus = 0; bus < inputs.size(); ++bus)
        require(processor->getBusArrangement(kInput, bus, inputs[bus]) == kResultOk, "Input arrangement query failed");
    for (std::size_t bus = 0; bus < outputs.size(); ++bus)
        require(processor->getBusArrangement(kOutput, bus, outputs[bus]) == kResultOk, "Output arrangement query failed");
    require(!outputs.empty() && outputs[0] == SpeakerArr::kStereo, "Default main output is not stereo");
    require(processor->setBusArrangements(inputs.data(), inputs.size(), outputs.data(), outputs.size()) == kResultOk,
            "Existing bus arrangement failed");
    for (int bus = 0; bus < component->getBusCount(kAudio, kOutput); ++bus)
        component->activateBus(kAudio, kOutput, bus, bus == 0);
    component->activateBus(kEvent, kInput, 0, true);
    ProcessSetup setup {kOffline, kSample32, block, rate};
    require(processor->setupProcessing(setup) == kResultOk, "setupProcessing failed");
    require(component->setActive(true) == kResultOk, "setActive failed");
    require(processor->setProcessing(true) == kResultOk, "setProcessing failed");
    HostProcessData data;
    require(data.prepare(*component, block, kSample32), "Process buffers failed");
    require(data.numOutputs >= 1 && data.outputs[0].numChannels == 2, "Main output is not stereo");
    data.processMode = kOffline;
    auto events = owned(new EventList());
    data.inputEvents = events;
    std::ostream output(&outputFile), trace(traceFile.get());
    require(output.good() && trace.good(), "Cannot create outputs");
    output.write("RIFF", 4); little(output, 36 + static_cast<uint32_t>(frames * 8));
    output.write("WAVEfmt ", 8); little(output, 16); little(output, 3, 2); little(output, 2, 2);
    little(output, rate); little(output, rate * 8); little(output, 8, 2); little(output, 32, 2);
    output.write("data", 4); little(output, static_cast<uint32_t>(frames * 8));
    trace << "frame,note,midi_velocity,vst3_float_velocity,float_times_127,nextafter\n";
    trace.precision(17);
    std::size_t next = 0;
    double peak = 0;
    for (int64_t frame = 0; frame < frames; frame += block) {
        data.numSamples = static_cast<int32>(std::min<int64_t>(block, frames - frame));
        events->clear();
        for (int bus = 0; bus < data.numInputs; ++bus) {
            data.inputs[bus].silenceFlags = HostProcessData::kAllChannelsSilent;
            for (int channel = 0; channel < data.inputs[bus].numChannels; ++channel)
                std::fill_n(data.inputs[bus].channelBuffers32[channel], data.numSamples, 0.0f);
        }
        for (int bus = 0; bus < data.numOutputs; ++bus) {
            data.outputs[bus].silenceFlags = 0;
            for (int channel = 0; channel < data.outputs[bus].numChannels; ++channel)
                std::fill_n(data.outputs[bus].channelBuffers32[channel], data.numSamples, 0.0f);
        }
        while (next < messages.size() && messages[next].frame < frame + data.numSamples) {
            const auto message = messages[next++];
            Event event {};
            event.busIndex = 0; event.sampleOffset = static_cast<int32>(message.frame - frame);
            event.type = message.on ? Event::kNoteOnEvent : Event::kNoteOffEvent;
            if (message.on) {
                float velocity = static_cast<float>(message.velocity) / 127.0f;
                if (roundUp && velocity < 1.0f) velocity = std::nextafter(velocity, 1.0f);
                event.noteOn = {0, static_cast<int16>(message.note), 0.0f, velocity, 0, -1};
                trace << message.frame << ',' << message.note << ',' << message.velocity << ','
                      << velocity << ',' << static_cast<double>(velocity) * 127.0 << ',' << roundUp << '\n';
            } else {
                event.noteOff = {0, static_cast<int16>(message.note), 0.0f, -1, 0.0f};
            }
            require(events->addEvent(event) == kResultOk, "Event queue failed");
        }
        require(processor->process(data) == kResultOk, "Audio processing failed");
        for (int sample = 0; sample < data.numSamples; ++sample) {
            for (int channel = 0; channel < 2; ++channel) {
                float value = data.outputs[0].channelBuffers32[channel][sample];
                require(std::isfinite(value), "Non-finite output");
                peak = std::max(peak, static_cast<double>(std::abs(value)));
                output.write(reinterpret_cast<char*>(&value), sizeof(value));
            }
        }
    }
    processor->setProcessing(false); component->setActive(false);
    require(peak > 0 && output.good() && trace.good(), "Silent render or output write failed");
    outputFile.finish(); traceFile->finish();
    outputFile.commit(); traceFile->commit();
    std::cout << "Rendered " << frames << " frames; peak=" << peak
              << "; diagnostic nextafter=" << roundUp << '\n';
    return 0;
} catch (const std::exception& error) {
    std::cerr << error.what() << '\n'; return 1;
}
