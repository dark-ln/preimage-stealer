# preimage-stealer

A utility to connect to claim HTLCs that we have already seen the preimage for.
This will automatically execute things like wormhole attacks for the user, allowing them to get free bitcoin.

Currently, this works by connecting to a lnd node and subscribes to the HTLC events to get preimages to save and the
HTLC interceptor to execute the theft.