m = 0
o = 0

slew_rate = 0.005
ringing = 0.05
samplerate = 48000

r = []
s = []
for i in range(samplerate * 5):
	v = int((i/(samplerate/1))%1>=.5)*2 - 1
	m += (v - o - m) / (1 + ringing * samplerate)
	o += m / (1 + slew_rate * samplerate)
	r.append(o)
	s.append(v)

# Numpy implementation
t = []
...

# Graphs

import pylab as pl
fig, axs = pl.subplots(1)
axs.plot(s, label="input")
axs.plot(r, label="output")
axs.legend()
pl.waitforbuttonpress()
pl.close()
