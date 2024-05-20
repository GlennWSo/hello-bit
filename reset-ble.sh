!#/usr/bin/env bash

read -p "Are you sure? " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    probe-rs erase --chip nrf52833_xxAA --allow-erase-all
    probe-rs download --verify --format hex --chip nRF52833_xxAA softdevice/s113_nrf52_7.3.0/s113_nrf52_7.3.0_softdevice.hex

fi

