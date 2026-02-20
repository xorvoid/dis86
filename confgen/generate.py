#!/usr/bin/env python3
import sys
import argparse
from hydra.gen import load, appdata, dis86

def run():
    parser = argparse.ArgumentParser(description='Generate annotation tables')
    parser.add_argument('source', type=str, help='input source annotations file')
    parser.add_argument('--output-path', type=str, help='path to write the generated output')
    parser.add_argument('--appdata-hdr', action='store_true', help='generate appdata header')
    parser.add_argument('--appdata-src', action='store_true', help='generate appdata source')
    parser.add_argument('--dis86-conf', action='store_true', help='generate dis86 configuration file')

    args = parser.parse_args()

    data = load.annotations(args.source)

    if args.output_path and args.output_path != '-':
        with open(args.output_path, 'w') as f:
            generate(args, data, f)
    else:
        generate(args, data, sys.stdout)

def generate(args, data, out):
    if args.appdata_hdr:
        appdata.gen_hdr(data, out)

    elif args.appdata_src:
        appdata.gen_src(data, out)

    elif args.dis86_conf:
        dis86.gen_conf(data, out)

run()
