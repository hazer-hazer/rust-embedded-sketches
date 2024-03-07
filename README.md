## Debugging

I've struggled with it for a long time...
So, we need a debugger, in my case I'll use Picoprobe, this is a Raspberry Pi Pico flashed with debugging firmware called Picoprobe too. We need to do that once, and it's possible just to find some picoprobe.uf2 file and drag-and-drop it onto rp-pico mass storage. 

> Note: Target board need to support debug connection, i.e. some pins with names such as SWCLK and SWIO. Waveshare RP2040-Zero does not have such pins thus it's not possible to debug it, and can be used as debug target.

Tutorial with needed software and Picoprobe -> connection graph can be found [here](https://reltech.substack.com/p/getting-started-with-rust-on-a-raspberry). Anyway I'll write down the most important steps below:

1. Install libs from the tutorial.
2. Build your own openocd, because `apt` software channel includes the version 0.10.x without rp2040 target. And it would be better to know what we are using. So clone the repo from Raspberry Pi GitHub on branch `picoprobe` which includes `picoprobe.cfg` for openocd. Do the installation steps from the tutorial.
3. Now you'll have openocd with picoprobe config on your hands (btw you can replace/set it as your default openocd, but I won't do it for clarity even I'm only using it for Pico only). If you don't want to make this openocd your default openocd, don't do `make install`. After build it will be placed in `{path to raspberry pi openocd repo}/src/openocd`. I'd added it to my path or as a new alias function to `.zshrc`/`.bashrc` for convenience. 
4. `picoprobe.uf2` binary can be found [here (official raspberry pi site)](https://datasheets.raspberrypi.org/soft/picoprobe.uf2) or on [GitHub](https://github.com/raspberrypi/debugprobe/releases). Note: Choose `picoprobe.uf2`, not `debugprobe.uf2`, second one is for a special board RP foundation manufactures.
5. If you're using WSL (as I do), you'll home some pain dealing with USB devices.
    - Install [usbipd for windows](https://github.com/dorssel/usbipd-win/releases)
    - Open powershell, do `usbipd wsl list` to see connected devices. Picoprobe should appear as `Picoprobe interface`
    - Do `usbipd wsl attach --busid {busid}` (you see busid for each device in the list shown above).
    - DO THAT EACH TIME CONNECTING USB DEVICE. Or be a lazy programmer and automate this (as I'll do someday).
    - To disconnect device, just do `usbipd wsl detach --busid {busid}`.
6. For WSL and Linux too, do user device access steps from tutorial, or you'll to do `chown` for yourself every time you connect your picoprobe.

Some tips:
- Use `sudo` for `openocd` if you have some problems with usb connection permissions.


Help links:
- [raspberry-pi-pico-cant-debug](https://community.platformio.org/t/raspberry-pi-pico-cant-debug/31794/16)
