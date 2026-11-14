# Instructions

0x0000 - NOP
0x1000 - Clear Frame Buffer
0x20XX - Draw color XX at the current cursor position
0x210x - Draw color in register X at the current cursor position
0x2E0x - Draw sprite in register X at the current cursor position
0x2FXX - Draw sprite XX at the current cursor position
0x3nnn - Move cursor to position nnn (Top 6 bits are the X coordinate, bottom 6 bits are the Y coordinate)
0x40kk - Increment X position by kk pixels
0x41kk - Decrement X position by kk pixels
0x42kk - Increment Y position by kk pixels
0x43kk - Decrement Y position by kk pixels
0x4A0x - Set cursor X position to value in register x
0x4A1x - Set cursor Y position to value in register x
0x5nnn - Jump to address nnn
0x6nxy - Skip next instruction if register X <cond> register Y holds (where <cond> is encoded in n)
0x7xkk - Move value kk into register x
0x80xy - Move value in register y into register x
0x9nnn - Call subroutine at address nnn
0xA000 - Return from subroutine
0xB00k - Skip next instruction if key k is pressed
0xC0xy - Add value in register y to register x and store the result in register x
0xC1xy - Subtract value in register y from register x and store the result in register x
0xC2xy - Multiply value in register x by value in register y and store the result in register x
0xC3xy - Divide value in register x by value in register y and store the result in register x
0xC4xy - Modulo value in register x by value in register y and store the result in register x
0xC5xy - Bitwise AND value in register x with value in register y and store the result in register x
0xC6xy - Bitwise OR value in register x with value in register y and store the result in register x
0xC7xy - Bitwise XOR value in register x with value in register y and store the result in register x
0xC8xy - Bitwise NOT value in register x and store the result in register x and store the result in register x
0xC9xy - Bitwise left shift value in register x by value in register y
0xCAxy - Bitwise right shift value in register x by value in register y
0xCBxy - Rotate value in register x left by value in register y
0xCCxy - Rotate value in register x right by value in register y
0xD00x - Set register x to a random value between 0 and 255
0xEnnn - Set sprite index to address nnn
0xFFFF - Exit
