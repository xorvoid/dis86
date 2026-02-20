#!/usr/bin/env python3
import sys
import os
import argparse
from pathlib import Path

def parse_args():
    parser = argparse.ArgumentParser(
        description="Run hydra engine"
    )
    parser.add_argument(
        "--exe",
        required=True,
        type=Path,
        help="Path to the dos exe to run"
    )
    parser.add_argument(
        "--libhydrauser",
        required=True,
        type=Path,
        help="Path to the libhydrauser library"
    )
    parser.add_argument(
        "--state-path",
        type=Path,
        help="Path capture/restore state"
    )
    parser.add_argument(
        "--capture-addr",
        type=str,
        help="Address to capture state"
    )
    parser.add_argument(
        "--mode",
        required=True,
        type=str,
        help="Operation mode"
    )

    return parser.parse_args()

def configure_mode(args):
    if args.mode == 'capture':
        return f'capture|{args.capture_addr}|{args.state_path}'
    elif args.mode == 'restore':
        return f'restore|{args.state_path}'
    else:
        return 'normal'

def main():
    args = parse_args()

    # Resolve paths
    exe = args.exe.resolve()
    libhydrauser = args.libhydrauser.resolve()
    mount_d = exe.parent
    exec_cmd = exe.name

    # Compute repo-local paths
    thisdir = Path(__file__).resolve().parent
    dosbox = thisdir / "src/dosbox-x/src/dosbox-x"
    conf = thisdir / "conf/dosbox.conf"

    # Mode
    mode = configure_mode(args)

    # Run from the exe mount dir
    os.chdir(mount_d)

    # Run it!
    os.environ["SDL_AUDIODRIVER"] = "pulse"

    cmd = [
        str(dosbox),
        "-conf",       f"{conf}",
        "-hydra",      f"{libhydrauser}",
        "-hydra-conf", mode,
        "-c",          f"mount d {mount_d}",
        "-c",          "D:",
        "-c",          exec_cmd,
        "-c",          "exit"
    ]

    os.execvp(cmd[0], cmd)

if __name__ == '__main__':
    main()
