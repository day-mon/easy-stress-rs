# Easy Stress
![Supported Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-blue)
#### This is a WIP!!
This is a command line tool that allows you to stress test your CPU & GPU.

## Why?
The goal of this project was to gain knowledge on how GPU programming works, as well as to learn more about Rust. 
I also wanted to create a tool that would allow me to stress test my system components without having to install a bunch of different programs.
When I was looking for a tool like this, I found that most of the tools I found were either too complicated to use or didn't work on my system and didnt test all the components I wanted to test. 
So I decided to create my own tool.

## Features
- CPU stress testing using multiple threads 
- GPU stress testing using OpenCL 
- Customizable test duration and termination conditions
- Customizable test methods for CPU and GPU stress testing

## Dependencies
- Rust
- OpenCL Library (should be in your graphics drivers)
    - Windows:
      - Intel GPUs:
        - https://www.intel.com/content/www/us/en/search.html
      - AMD GPUs:
        - https://www.amd.com/en/support
      - Nvidia GPUs:
        - https://www.nvidia.com/Download/index.aspx?lang=en-us
    - Linux:
       - Debian/Ubuntu: `sudo apt install ocl-icd-opencl-dev`
       - Arch: `sudo pacman -S ocl-icd` or `yay/paru -S opencl-nvidia 525.85.05-1`
       - Fedora: `sudo dnf install ocl-icd-devel`
     - MacOS: From my testing on my M1 Mac, the OpenCL library is already installed. If you are using an Intel Mac I dont know and I am going to attempt to figure it out.

## Usage
#### Install from package managers coming soon!

To use the tool, first install Rust if you haven't already. Then, clone the repository and run the following command:

```bash
cargo run
```
- You will be prompted to select a component to stress test and configure the test settings. The tool will then run the stress test and display the results. 
- You can also build the project with the following command:
```bash
cargo build --release
cd target/release
./easy-stress-rs
```

### Known Issues
- Sometimes the tool will not compile on Windows because it fails to find the OpenCL library because x86_64-pc-windows-msvc uses .lib and not .dlls. To fix this open your finder and look for OpenCL.lib. Then copy it to the target/release/deps folder, then attempt to recompile.
- On M1 Macbooks the tool will recognize the GPU but will not be able to stress test it. I am working on a fix for this.
- On some GPUs the tool will not be able to stress test them. I am working on a fix for this. 

### Note
- The tool has been tested on Windows and Linux and should also work on macOS.
- The tool will automatically detect and use all available GPUs, but you must have the appropriate OpenCL drivers installed.
- Also, please note that stress testing can potentially damage your hardware. Use the tool at your own risk and make sure to monitor the temperature and performance of your system while the test is running.