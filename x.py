import subprocess;
import sys;
import os;

subcommand = sys.argv[1];


if subcommand == "test":
    if len(sys.argv) >= 3:
        dir = sys.argv[2]
    else:
        dir = "tests"
    p = subprocess.Popen(["cargo","-q","r","--","test","../../{dir}".format(dir = dir),"../jessie"],cwd="./crates/jessie-test")
    p.wait()

if subcommand == "bless":
    if len(sys.argv) >= 3:
        dir = sys.argv[2]
    else:
        dir = "tests"
    p = subprocess.Popen(["cargo","-q","r","--","test","../../{dir}".format(dir = dir),"../jessie","--bless"],cwd="./crates/jessie-test")
    p.wait()
