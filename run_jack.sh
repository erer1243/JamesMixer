#!/usr/bin/env bash
kill_jack_procs() {
	pkill -x jackd
	pkill -x alsa_in
}

output() {
	HOSTNAME="$(hostname)"
	[ "$HOSTNAME" = computinator ] && echo "hw:Audio"
	[ "$HOSTNAME" = silk ]         && echo "hw:PCH"
}

# Ensure no jack server is already running
kill_jack_procs

# Start jackd
jackd                                   \
	-d alsa  `# Alsa backend`       \
	-r 48000 `# Sample rate`        \
	-p 512   `# Frames per period`  \
	-n 3     `# Periods per buffer` \
	-D       `# Duplex mode`        \
	-C hw:Microphone `# Mic input`  \
	-P "$(output)"   `# Speaker output` &

# Wait for jackd to be up and running
sleep 0.2

# Start alsa_in with line in input
alsa_in -j line_in -d hw:Device &

# Start qjackctl for port connection management
qjackctl -s

# After qjackctl is closed, kill jack processes
kill_jack_procs
wait
