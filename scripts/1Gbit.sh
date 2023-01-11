fichier=1Gbit # soit 1 000 000 000 bit soit 125 000 000 Bytes
rm ${fichier} ${fichier}.torrent

## 1. Créer un fichier de 32000K : 1K = 1Kio = 1024 B
# https://fr.wikipedia.org/wiki/Octet#Multiples_normalis%C3%A9s
# man gdd : K=1024

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     dd=dd;; # utiliser dd pour Linux
    Darwin*)    dd=gdd;; #utiliser gdd pour MacOS (brew install coreutils)
    *)          machine="UNKNOWN:${unameOut}"
esac


$dd if=/dev/random bs=1MB count=125 of=${fichier} 
# ls -l 125000000 car bytes
#  % gls -l --block-size=MB 1Gbit
# -rw-r--r-- 1 alphand wheel 125MB Jan  9 16:39 1Gbit

## 2. create torrent
# 2.1 compile mktorrent previously
# git clone https://github.com/pobrn/mktorrent.git
# cd mktorrent
# make
# 2.2 create torrent

# make sure mktorrent is in your path variable
# ./mktorrent/mktorrent -v -p -a http://127.0.0.1:6969/announce -o 10Gbit.torrent 10Gbit
mktorrent -v -p -a http://127.0.0.1:6969/announce -o ${fichier}.torrent ${fichier}
# 125000000 bytes in all
# that's 954 pieces of 131072 bytes each

