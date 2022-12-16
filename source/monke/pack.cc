// codeshaunted - monke
// source/monke/pack.cc
// contains definitions for Pack class
// Copyright 2022 codeshaunted
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

#include "pack.hh"

#include <fstream>
#include <iostream>

#include "cryptopp/cryptlib.h"
#include "cryptopp/pwdbased.h"
#include "cryptopp/sha.h"
#include "cryptopp/rijndael.h"
#include "cryptopp/aes.h"
#include "cryptopp/modes.h"
#include "cryptopp/filters.h"
#include "cryptopp/files.h"
#include "cryptopp/zlib.h"
#include "cryptopp/osrng.h"

#include "config.hh"

namespace monke {

void Pack::pack(std::string_view input_path, std::string_view output_path, std::string_view password) {
  // open output file
  std::ofstream output_file;
  output_file.open(output_path.data(), std::ios::out | std::ios::binary);

  // write dummy header
  char dummy_header[MONKE_DUMMY_HEADER_LENGTH] = {0};
  output_file.write(dummy_header, sizeof(dummy_header));

  // write password index
  uint64_t password_index = MONKE_PASSWORD_INDEX;
  output_file.write(reinterpret_cast<char*>(&password_index), sizeof(password_index));

  // write salt
  CryptoPP::SecByteBlock salt(MONKE_SALT_LENGTH);
  CryptoPP::OS_GenerateRandomBlock(true, salt, salt.size());
  output_file.write(reinterpret_cast<char*>(salt.BytePtr()), salt.size());

  // derive key and iv from salt and password
  CryptoPP::byte derived_key[MONKE_KEY_LENGTH + MONKE_IV_LENGTH];
  CryptoPP::PKCS5_PBKDF2_HMAC<CryptoPP::SHA1> pbkdf2;
  pbkdf2.DeriveKey(derived_key, MONKE_KEY_LENGTH + MONKE_IV_LENGTH, 0, (const CryptoPP::byte*)password.data(), password.size(), salt.BytePtr(), salt.size(), MONKE_DERIVE_ITERATIONS, 0.0f);

  // set key and iv
  CryptoPP::CBC_Mode<CryptoPP::AES>::Encryption encryptor;
  encryptor.SetKeyWithIV(derived_key + MONKE_IV_LENGTH, MONKE_KEY_LENGTH, derived_key, MONKE_IV_LENGTH);

  // open input file
  std::ifstream input_file;
  input_file.open(input_path.data(), std::ios::in | std::ios::binary);

  // compress data
  std::vector<CryptoPP::byte> compressed_data;
  CryptoPP::FileSource file_source(input_file, true, new CryptoPP::ZlibCompressor(new CryptoPP::VectorSink(compressed_data), MONKE_COMPRESSION_LEVEL, MONKE_COMPRESSION_LOG2_BUFFER_SIZE));

  // close input file and flush output file
  input_file.close();
  output_file.flush();

  // encrypt data and write to output file
  CryptoPP::VectorSource vector_source(compressed_data, true, new CryptoPP::StreamTransformationFilter(encryptor, new CryptoPP::FileSink(output_file)));

  // close output file
  output_file.close();
}

void Pack::unpack(std::string_view input_path, std::string_view output_path, std::string_view password) {
  // open input file
  std::ifstream input_file;
  input_file.open(input_path.data(), std::ios::in | std::ios::binary);

  // read dummy header (44 bytes of garbage?)
  char dummy_header[MONKE_DUMMY_HEADER_LENGTH];
  input_file.read(dummy_header, sizeof(dummy_header));

  // read password index; not needed for Bloons TD 6
  uint64_t password_index;
  input_file.read(reinterpret_cast<char*>(&password_index), sizeof(password_index));

  // read salt
  char salt[MONKE_SALT_LENGTH];
  input_file.read(salt, sizeof(salt));

  // derive key and iv from salt and password
  CryptoPP::byte derived_key[MONKE_KEY_LENGTH + MONKE_IV_LENGTH];
  CryptoPP::PKCS5_PBKDF2_HMAC<CryptoPP::SHA1> pbkdf2;
  pbkdf2.DeriveKey(derived_key, MONKE_KEY_LENGTH + MONKE_IV_LENGTH, 0, (const CryptoPP::byte*)password.data(), password.size(), (const CryptoPP::byte*)salt, sizeof(salt), MONKE_DERIVE_ITERATIONS, 0.0f);

  // set key and iv
  CryptoPP::CBC_Mode<CryptoPP::AES>::Decryption decryptor;
  decryptor.SetKeyWithIV(derived_key + MONKE_IV_LENGTH, MONKE_KEY_LENGTH, derived_key, MONKE_IV_LENGTH);

  // open output file
  std::ofstream output_file;
  output_file.open(output_path.data(), std::ios::out | std::ios::binary);

  // decrypt data
  std::vector<CryptoPP::byte> decrypted_data;
  CryptoPP::FileSource file_source(input_file, true, new CryptoPP::StreamTransformationFilter(decryptor, new CryptoPP::VectorSink(decrypted_data)));
  
  // decompress data and write to output file
  CryptoPP::VectorSource array_source(decrypted_data, true, new CryptoPP::ZlibDecompressor(new CryptoPP::FileSink(output_file)));

  // close file streams
  input_file.close();
  output_file.close();
}

} // namespace monke
