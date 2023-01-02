#!/usr/bin/env bash

rm -rf aria* *[0-9].jpg # supprimer tout sauf le script

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     dd=dd;; # utiliser dd pour Linux
    Darwin*)    dd=gdd;; #utiliser gdd pour MacOS (brew install coreutils)
    *)          machine="UNKNOWN:${unameOut}"
esac


# Sous MacOS
# which dd
# /bin/dd
# brew install coreutils #installe gdd dans /usr/local/bin 
# qui pointe sur /usr /local/Cellar/coreutils/9.1/bin/gdd
# which gdd
#/bin/dd
# gdd 
# alphand@thera partial % aria2c -S ../iceberg.jpg.torrent
# Piece Length: 32KiB
# The Number of Pieces: 11
# Total Length: 348KiB (356,639)
# https://fr.wikipedia.org/wiki/Octet 1Ko = 1000B et 2^10 = 1KiB (kibioctet)
# pièces de 32768B ou 32KiB ou 2<<15 (jshell)

# 4 1ères pièces de 32Ko
$dd if=iceberg.jpg of=./iceberg1-4.jpg bs=32KiB count=4 

# 4 pièces du milieu
# 1. d'abord un fichier plein de zero
# 2. ajout des 4x32Ko après les 4 x 32Ko
# 3. vérif écriture à 2<<16 (131072 ou 0x0020000) jusqu'à 2<<18 (262144 ou 0x0040000)
$dd if=/dev/zero of=iceberg5-8.jpg bs=1 count=356639
$dd conv=notrunc bs=32KiB count=4 if=iceberg.jpg skip=4 of=iceberg5-8.jpg seek=4

# 3 pièces à la fin
$dd if=/dev/zero of=iceberg9-11.jpg bs=1 count=356639
$dd conv=notrunc bs=32KiB count=3 if=iceberg.jpg skip=8 of=iceberg9-11.jpg seek=8
# ou
# gdd if=/dev/zero of=iceberg9-11.jpg bs=1 count=356639
# gdd conv=notrunc bs=1 count=94495 if=../iceberg.jpg skip=262144 of=iceberg9-11.jpg  seek=262144

# Test du resume avec aria

mkdir -p aria1-4
cp iceberg1-4.jpg aria1-4/iceberg.jpg
mkdir -p aria5-8
cp iceberg5-8.jpg aria5-8/iceberg.jpg
mkdir -p aria9-11
cp iceberg9-11.jpg aria9-11/iceberg.jpg
#
# aria="aria2c --disable-ipv6=true --enable-dht=false -V"
#
# $aria -d aria1-4 iceberg.jpg.torrent
# #c2ece1 128KiB/348KiB(36%)
# $aria -d aria5-8 iceberg.jpg.torrent
# #d5df5e 128KiB/348KiB(36%)
# $aria -d aria9-11 iceberg.jpg.torrent
# #9d3888 92KiB/348KiB(26%)

# EXPLICATIONS puis EXEMPLE
# https://unix.stackexchange.com/questions/146922/is-dd-able-to-overwrite-parts-of-a-file
# conv=notrunc ne pas tronquer le fichier là où finit le dd # 
#
# https://stackoverflow.com/questions/41955325/bash-how-to-write-a-file-to-a-specific-address-on-a-disk
# bs=BYTES
# read and write up to BYTES bytes at a time
# count=N copy only N input blocks
# seek=N skip N obs-sized blocks at start of output
# skip=N skip N ibs-sized blocks at start of input
#
# https://superuser.com/questions/380717/how-to-output-file-from-the-specified-offset-but-not-dd-bs-1-skip-n
# count_bytes # Interpret the `count=' operand as a byte count
# skip_bytes # Interpret the `skip=' operand as a byte count
# seek_bytes # Interpret the `seek=' operand as a byte count
#
# EXEMPLE
# gdd if=/dev/random of=random.bin bs=16 count=4
# gdd if=/dev/zero of=one.bin bs=16 count=4
# gdd conv=notrunc bs=16 count=2 if=random.bin skip=1 of=one.bin seek=1
# hexdump one.bin
# 0000000 0000 0000 0000 0000 0000 0000 0000 0000
# 0000010 3d6e cd5e 39a2 5fa3 860c 53ac 50ae 9cca
# 0000020 b353 a16c f5d8 98ac be9d 07b4 88b9 54d9
# 0000030 0000 0000 0000 0000 0000 0000 0000 0000
# 0000040