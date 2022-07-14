#!/bin/bash

BUILD_TYPE=""
#BUILD_TYPE="-DCMAKE_BUILD_TYPE=Debug"

echo If running debian/ubuntu install dependecies with the following command:
echo sudo apt install clang llvm gcc libssl-dev python3-pytest python3-pytest-xdist unzip xsltproc doxygen graphviz
if [[ x$1 != x"--yes" ]]; then
    read -p "Press enter to continue"
fi

cd liboqs

mkdir -p build && cd build
cmake .. $BUILD_TYPE -DOQS_USE_OPENSSL=no -DOQS_MINIMAL_BUILD="OQS_ENABLE_KEM_hqc_128;OQS_ENABLE_KEM_hqc_192;OQS_ENABLE_KEM_hqc_256;OQS_ENABLE_KEM_bike_l1;OQS_ENABLE_KEM_bike_l3;OQS_ENABLE_KEM_frodokem_640_aes;OQS_ENABLE_KEM_frodokem_640_shake;OQS_ENABLE_KEM_frodokem_976_aes;OQS_ENABLE_KEM_frodokem_976_shake;OQS_ENABLE_KEM_frodokem_1344_aes;OQS_ENABLE_KEM_frodokem_1344_shake;OQS_ENABLE_KEM_kyber_512;OQS_ENABLE_KEM_kyber_768;OQS_ENABLE_KEM_kyber_1024;OQS_ENABLE_KEM_kyber_512_90s;OQS_ENABLE_KEM_kyber_768_90s;OQS_ENABLE_KEM_kyber_1024_90s" -DOQS_BUILD_ONLY_LIB=ON
make -j `nproc`

# Ensure rebuild of liboqs-rs-bindings:
touch wrapper.h