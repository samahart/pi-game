# pi-game
A fun game to test your ability to recite the digits of pi

## Project setup

### Install DietPi

- Download image for Pi 3B from: https://dietpi.com/downloads/images/DietPi_RPi234-ARMv8-Bookworm.img.xz
- Flash to SD card using Raspberry Pi's imager: https://downloads.raspberrypi.org/imager/imager_latest.exe
- Follow on screen prompts on imager and select "Use custom" for the OS, then select the downloaded DietPi image.
- Plug SD card into Pi and complete installation process on screen.

### Once logged into Pi

Run `dietpi-software` and install Python. Next, install the following libs:

```
alsa-utils
libportaudio2
libatomic1
python3.11-venv
```

Run `dietpi-config` and enable the Audio Options. For my Logitech headset (with USB adapter), I had to enable the following settings to get the headset and mic to work:

```
soundcard: (hw:0,0)
auto conversion: on
```

#### Test speakers

```
speaker-test -t wav -c 2
```

You should hear audio alternating to each ear.

#### Test microphone

```
# record for 5 seconds
arecord -d 5 -V mono -f S16_LE -r 16000 record_test.wav
```

```
# playback
aplay record_test.wav
```
