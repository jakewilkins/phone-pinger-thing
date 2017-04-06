phone pinger thing
==================

I have a pythong script that notices when my phone connects to wifi.

1. I'm bad at python.
2. I don't know how to extend it to just scan for traffic and notice after it stops for a while.
3. I _kind of_ know how to write a Rust program that pings it and notices after a certain number of ping failures.

So this is that. It pings the IP that the python script sets in Redis. If it is
silent more than max_misses then we assume I'm no longer at home.

I'v been told I should be using bluetooth scanning for this, but I have some
apparently bad coverage for bluetooth around my house because I found that extremely
unreliable.


