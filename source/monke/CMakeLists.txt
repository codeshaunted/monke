# averysumner - monke
# CMakeLists.txt
# source file directory CMake file
# Copyright 2021 averysumner
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http:#www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set(MONKE_SOURCE_FILES
	"${CMAKE_CURRENT_SOURCE_DIR}/main.cc"
	"${CMAKE_CURRENT_SOURCE_DIR}/pack.cc")

set(MONKE_INCLUDE_DIRECTORIES
	"${CMAKE_SOURCE_DIR}/include/monke"
	"${CMAKE_SOURCE_DIR}/third_party/cryptopp-cmake"
	"${PROJECT_BINARY_DIR}/source/monke")

set(MONKE_LINK_LIBRARIES
	cryptopp-static)

set(MONKE_COMPILE_DEFINITIONS
	NDEBUG)

configure_file("${CMAKE_SOURCE_DIR}/include/monke/config.hh.in" "config.hh")

add_executable(monke ${MONKE_SOURCE_FILES})

target_include_directories(monke PUBLIC ${MONKE_INCLUDE_DIRECTORIES})

target_link_libraries(monke PUBLIC ${MONKE_LINK_LIBRARIES})

target_compile_definitions(monke PUBLIC ${MONKE_COMPILE_DEFINITIONS})