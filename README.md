# pi-game
A fun game to test your ability to recite the digits of pi

## Raspberry Pi Setup

### Install DietPi

- Download the associated image for your model Pi from: https://dietpi.com/#download 
- Flash to SD card using Raspberry Pi's imager: https://downloads.raspberrypi.org/imager/imager_latest.exe
- Follow on screen prompts on imager and select "Use custom" for the OS, then select the downloaded DietPi image.
- Plug SD card into Pi and complete installation process on screen.

### Setup DietPi

Once initial setup is complete, run `dietpi-software` and install Python. Next, install the following libs from the terminal:

```
apt install alsa-utils libportaudio2 libatomic1 python3.11-venv
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

## Build pi-game

This section details how to setup and compile the project. Additionally it provides steps to link the shared libraries needed to build/run the project. Note, development for this project was done in both Windows and Linux so build steps are included for both operating systems.

#### Shared Libraries

Download the appropriate Arch zip file from https://github.com/alphacep/vosk-api/releases/tag/v0.3.45. Extract it to the desired location on the system.

Before running the build step, point the Rust environment variable to where the shared lib is located. For example on Linux:

```
TODO
```

On Windows Powershell:

```powershell
$env:RUSTFLAGS="-L C:\path\to\vosk-win64-0.3.45"
$env:PATH += ";C:\path\to\vosk-win64-0.3.45"
```


### Build

Obtain the source code for pi-game:

```bash
git clone git@github.com:samahart/pi-game.git
```

Install Rust following the instructions here: https://www.rust-lang.org/tools/install. Once Rust is installed, you're ready to test if pi-game builds without any errors. From the root directory of pi-game run:

```bash
cargo build
```

## Run pi-game

Before you can run pi-game, you must download the Vosk model. See the next step for instructions on obtaining the speech-to-text model.

### Models

Vosk models can be downloaded here: https://alphacephei.com/vosk/models

To run on the Pi, the lightweight "vosk-model-small-en-us-0.15" model must be used. Download the model and extract it somewhere on your filesystem. The directory where the model is located will be passed in as a commandline parameter to the pi-game executable.

### Run

Run the following command from your terminal to execute pi-game:

```bash
cargo run -- /path/to/vosk-model-small-en-us-0.15
```
