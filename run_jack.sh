#!/usr/bin/env bash
kill_jack_procs() {
	pkill -x jackd
	pkill -x alsa_in
}

# Ensure no jack server is already running
kill_jack_procs

# Start jackd
jackd                                   \
	-d alsa  `# Alsa backend`       \
	-r 48000 `# Sample rate`        \
	-p 512   `# Frames per period`  \
	-n 2     `# Periods per buffer` \
	-D       `# Duplex mode`        \
	-C hw:Microphone `# Mic input`  \
	-P hw:PCH        `# Speaker output` &

# Wait for jackd to be up and running
sleep 1

# Start alsa_in with line in input
alsa_in -j line_in -d hw:Device &

# Start qjackctl for port connection management
qjackctl -s

# After qjackctl is closed, kill jack processes
kill_jack_procs
wait
