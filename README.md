# pi-game
A fun game to test your ability to recite the digits of pi

## Build pi-game

This section details how to setup and compile the project. Additionally it provides steps to link the shared libraries needed to build/run the project. Note, development for this project was done in both Windows and Linux so build steps are included for both operating systems.

### Shared Libraries

Download the appropriate Arch zip file from https://github.com/alphacep/vosk-api/releases/tag/v0.3.45. Extract it to the desired location on the system.

Before running the build step, point the Rust environment variable to where the shared lib is located. For example on Linux:

```bash
export RUSTFLAGS="-L /path/to/vosk-linux-aarch64-0.3.45"
export LD_LIBRARY_PATH="/path/to/vosk-linux-aarch64-0.3.45"
```

On Windows Powershell:

```powershell
$env:RUSTFLAGS="-L C:\path\to\vosk-win64-0.3.45"
$env:PATH += ";C:\path\to\vosk-win64-0.3.45"
```

### Build

Obtain the source code for pi-game:

```bash
git clone https://github.com/samahart/pi-game.git
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

## Raspberry Pi Setup

### Install DietPi

- Download the associated image for your model Pi from: https://dietpi.com/#download 
- Flash to SD card using Raspberry Pi's imager: https://downloads.raspberrypi.org/imager/imager_latest.exe
- Follow on screen prompts on imager and select "Use custom" for the OS, then select the downloaded DietPi image.
- Plug SD card into Pi and complete installation process on screen.

### Setup DietPi

Run `dietpi-config` and enable the Audio Options. For my Logitech headset (with USB adapter), I had to enable the following settings to get the headset and mic to work:

```bash
soundcard: (hw:0,0)
auto conversion: on
```

### Setup Game

Run the following commands to download the Vosk shared libraries and model.

```bash
mkdir /home/dietpi/vosk
cd vosk
curl -LO https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-aarch64-0.3.45.zip
unzip vosk-linux-aarch64-0.3.45.zip
curl -LO https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip
unzip vosk-model-small-en-us-0.15.zip
```

Add the following lines to file in `/home/dietpi/.bashrc` (note, you may need to install a text editor such as vim):

```bash
export RUSTFLAGS="-L /home/dietpi/vosk/vosk-linux-aarch64-0.3.45"
export LD_LIBRARY_PATH="/home/dietpi/vosk/vosk-linux-aarch64-0.3.45"
```

Install Rust/build dependencies and build game:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source /home/dietpi/.bashrc
cd
git clone https://github.com/samahart/pi-game.git
cd pi-game
apt install build-essential pkg-config libasound2-dev
cargo build --release
```

Before running the game, you need to add yourself to the audio group for permission to use the microphone:

```bash
sudo usermod -aG audio dietpi
sudo reboot
```

### Run Game

```bash
cargo run --release -- /home/dietpi/vosk/vosk-model-small-en-us-0.15
```

### Debugging Sound I/O

If you are unsure if the microphone connected to the Pi is working you can try to record some audio and play it back. Run the following command to install tools for debugging the microphone on the Pi.

```bash
apt install alsa-utils
```

#### Test speakers

```bash
speaker-test -t wav -c 2
```

You should hear audio alternating in each headphone/speaker.

#### Test microphone

```bash
# record for 5 seconds
arecord -d 5 -V mono -f S16_LE -r 16000 record_test.wav
# playback
aplay record_test.wav
```
