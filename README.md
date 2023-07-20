# Savi

A simple p2p voice call app made in Rust with Slint

I started this app as something to do while learning another programming language, apparently it was harder than I expected since I'm full of problems:

- High cpu usage at random moments
- My lack of understanding of rust multithreading results in weird hacky code
- Needs some serious deep code refactor


# TO BE ARCHIVED 
I've learned a lot of things while making this app but I'm being limited by my knowledge/experience with this ownership&borrowing thing in rust so i'll probably try something easier and leave this thing public as a proof that people should always learn the basics first lol, here are some really serious problems I'm facing rn:

- Something interrupts the client and server threads, which means only two people can connect at any moment (kinda beats the point of the thing being p2p)
- Unable to pass data to the gui from another thread (even after reading https://docs.rs/slint/latest/slint/fn.invoke_from_event_loop.html)
- Huge cpu usage when two people have a connection and a third tries to enter (the thing won't go down unless you close the app)

Maybe I'll try to make it again in a terminal app and work the gui from that
