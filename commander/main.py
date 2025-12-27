import argparse
import subprocess
import os
import sys

def main():
    parser = argparse.ArgumentParser(description="Scavenger Commander")
    parser.add_argument("--strategy", type=str, default="arb", help="Strategy to run (arb, sniper)")
    args = parser.parse_args()

    # Calculate paths
    # Assuming commander/main.py is located at root/commander/main.py
    commander_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(commander_dir)
    scavenger_dir = os.path.join(project_root, "scavenger")
    
    # Config path (absolute)
    config_path = os.path.join(commander_dir, "configs", f"{args.strategy}.yaml")

    if not os.path.exists(config_path):
        print(f"Error: Config file not found at {config_path}")
        sys.exit(1)

    # Rust binary path
    rust_binary = os.path.join(scavenger_dir, "target", "release", "scavenger")

    print(f"--- Scavenger Commander ---")
    print(f"Strategy: {args.strategy}")
    print(f"Config:   {config_path}")
    print(f"CWD:      {scavenger_dir}")

    cmd = []
    # Check if binary exists, otherwise use cargo run
    if os.path.exists(rust_binary):
        print(f"Binary found at {rust_binary}")
        cmd = [rust_binary, "--strategy", args.strategy, "--config", config_path]
    else:
        print(f"Binary not found, falling back to 'cargo run'...")
        # cargo run needs to be run in scavenger dir or with --manifest-path
        cmd = ["cargo", "run", "--release", "--bin", "scavenger", "--", "--strategy", args.strategy, "--config", config_path]

    print(f"Executing: {' '.join(cmd)}")
    print("-" * 30)

    try:
        # We set CWD to scavenger_dir because the config files use relative paths for keys (e.g. "auth_key.json")
        # which are expected to be in the scavenger directory.
        subprocess.run(cmd, cwd=scavenger_dir, check=True)
    except KeyboardInterrupt:
        print("\nStopping...")
    except subprocess.CalledProcessError as e:
        print(f"\nError: Process exited with code {e.returncode}")
        sys.exit(e.returncode)
    except Exception as e:
        print(f"\nError: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
