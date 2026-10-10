cmake_minimum_required(VERSION 3.25)
project(batcherbird_decent_vst3 LANGUAGES C CXX OBJC OBJCXX)
if(NOT VST3_SDK_ROOT)
  message(FATAL_ERROR "Pass -DVST3_SDK_ROOT=/path/to/official/vst3sdk")
endif()
set(SMTG_ENABLE_VST3_PLUGIN_EXAMPLES OFF CACHE BOOL "" FORCE)
set(SMTG_ENABLE_VST3_HOSTING_EXAMPLES OFF CACHE BOOL "" FORCE)
set(SMTG_ENABLE_VSTGUI_SUPPORT OFF CACHE BOOL "" FORCE)
add_subdirectory("${VST3_SDK_ROOT}" "${CMAKE_BINARY_DIR}/sdk")
add_executable(batcherbird_decent_vst3 main.cpp
  "${VST3_SDK_ROOT}/public.sdk/source/common/memorystream.cpp"
  "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/plugprovider.cpp"
  "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/module_mac.mm")
target_compile_features(batcherbird_decent_vst3 PRIVATE cxx_std_17)
target_compile_options(batcherbird_decent_vst3 PRIVATE "$<$<COMPILE_LANGUAGE:OBJCXX>:-fobjc-arc>")
target_include_directories(batcherbird_decent_vst3 PRIVATE "${VST3_SDK_ROOT}")
target_link_libraries(batcherbird_decent_vst3 PRIVATE sdk_hosting "-framework CoreFoundation" "-framework Cocoa")
