
![[Pasted image 20260815193658.png]]![[Pasted image 20260815193710.png|190]]


## Sources
- Semiconductor design solutions manual: https://caro.su/msx/ocm_de1/16550.pdf
- Open SoC page: https://opensocdebug.readthedocs.io/en/latest/02_spec/07_modules/dem_uart/uartspec.html
- Wikipedia: https://en.wikipedia.org/wiki/16550_UART
## Features
- Full-featured transmitter-receiver pair, configurable by software for different speeds, character widths, parity codification, etc.
- Converts serial to parallel, and parallel to serial, using [[shift registers]]
- On-chip bit rate (baud rate) generator to control transmit and receive data rate.
- Handshake lines for control of an external modem, controllable by software.
- Interrupt function to the host microprocessor
- on-chip FIFO buffer for both incoming and outgoing data
- DMA capability
- 

**Hardware and software interfaces of the 16550 are backward compatible with earlier 8250 and 16450 UART**