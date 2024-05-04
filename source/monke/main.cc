// codeshaunted - monke
// source/monke/main.cc
// contains entry point
// Copyright 2024 codeshaunted
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org / licenses / LICENSE - 2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissionsand
// limitations under the License.

#include <iostream>
#include <cstring>

#include "pack.hh" 
#include "config.hh"

void help() {
  std::cout << "monke v" << MONKE_VERSION_MAJOR << "." << MONKE_VERSION_MINOR << std::endl;
  std::cout << "Copyright 2024 codeshaunted" << std::endl << std::endl;
  std::cout << "Usage: monke <command> [arguments]" << std::endl << std::endl;
  std::cout << "Commands:" << std::endl;
  std::cout << " pack <input path> <output path> <password>" << std::endl;
  std::cout << " unpack <input path> <output path> <password>" << std::endl;
}

int main(int argc, char* argv[]) {
  if (argc == 1) {
    help();
    return 0;
  }

  if (!strcmp(argv[1], "unpack") || !strcmp(argv[1], "u")) {
    if (argc != 5) {
      help();
      return 0;
    }

    // cursed error handling
    // at least it doesn't silent fail anymore ¯\_(ツ)_/¯
    try {
      monke::Pack::unpack(argv[2], argv[3], argv[4]);
      std::cout << "Unpacked '" << argv[2] << "' with password '" << argv[4] << "' to '" << argv[3] << "'." << std::endl;
    } catch (...) {
      std::cout << "Failed to unpack '" << argv[2] << "' with password '" << argv[4] << "!" << std::endl;
    }
  }
  else if (!strcmp(argv[1], "pack") || !strcmp(argv[1], "p")) {
    if (argc != 5) {
      help();
      return 0;
    }

    try {
      monke::Pack::pack(argv[2], argv[3], argv[4]);
      std::cout << "Packed '" << argv[2] << "' with password '" << argv[4] << "' to '" << argv[3] << "'." << std::endl;
    } catch (...) {
      std::cout << "Failed to pack '" << argv[2] << "' with password '" << argv[4] << "!" << std::endl;
    }
  }
  else {
    help();
    return 0;
  }

  return 0;
}
