#!/bin/bash



tar -xf $1.tar.xz

cd $1

for dir in ./*
do
    zip -r $dir.sav $dir
    rm -r $dir
done
