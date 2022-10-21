# Getting started

## Requirements
- [McStatus](https://github.com/py-mine/mcstatus)
- [Rust](https://www.rust-lang.org/)
- [Masscan](https://github.com/robertdavidgraham/masscan)
- [Awk](https://www.gnu.org/software/gawk/)

## Installation
* Install the requirements
* clone the repo
* Edit `masscan.conf`
* Scan for servers **More info [here](#scanning-for-servers)**
* Rename `.env.example` to `.env` and edit it
* then run the project with `cargo run`

## Scanning for servers
### Ip list format
```
<ip>
<ip>
<ip>
<ip>
...
```
### Using masscan
just edit `masscan.conf` and run 
```
sudo masscan -c masscan.conf
```
then format the output with 
```
./format_masscan_output.sh
```