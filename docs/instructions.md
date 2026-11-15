# Instructions

0x0000 - NOP
0x020X - Load 8 bits from location pointed by index register into register rX
0x021X - Store 8 bits from register rX into location pointed by index register
0x022X - Load 16 bits from location pointed by index register into register rX
0x023X - Store 16 bits from register rX into location pointed by index register
0x024X - Move index register forward by value held in register rX
0x030X - Push value held in register rX onto the stack
0x031X - Pop value from the stack into register rX
0x04nk - Skip next instruction if flag number k is set to n
0x1000 - Clear the display
0x110X - Set timer counter to value held in register rX
0x111X - Set timer reload to value held in register rX
0x112X - Store timer counter into register rX
0x113k - Set timer configuration to k (bit 0: periodic, bit 1: enabled)
0x120k - Enable interrupt k
0x121k - Disable interrupt k
0x130k - Set interrupt vector for interrupt k to value held in index register
0x1400 - Enable interrupts
0x1401 - Disable interrupts
0x1500 - Wait
0x20KK - Draw color KK at cursor position
0x210X - Draw color held in register rX at cursor position
0x22KK - Draw sprite KK at cursor position
0x230X - Draw sprite held in register rX at cursor position
0x3KKK - Move cursor to position KKK (top 6 bits: X pos, bottom X bits: Y pos)
0x40KK - Increment cursor X by KK (wrapping to 0 if overflow)
0x41KK - Increment cursor Y by KK (wrapping to 0 if overflow)
0x42KK - Decrement cursor X by KK (wrapping to 0 if overflow)
0x43KK - Decrement cursor Y by KK (wrapping to 0 if overflow)
0x4A0X - Set cursor X to value held in register rX
0x4B0X - Set cursor Y to value held in register rX
0x5KKK - Jump to position KKK
0x60XY - Skip next instruction if value in rX == value in rY
0x61XY - Skip next instruction if value in rX != value in rY
0x62XY - Skip next instruction if value in rX < value in rY
0x63XY - Skip next instruction if value in rX > value in rY
0x64XY - Skip next instruction if value in rX <= value in rY
0x65XY - Skip next instruction if value in rX >= value in rY
0x7XKK - Move KK into register rX
0x80XY - Copy value in register rX into register rY
0x9KKK - Call subroutine at address KKK
0xA000 - Return from subroutine
0xA001 - Return from interrupt vector
0xB00K - Skip next instruction if key K is pressed
0xC0XY - Add value in register rX with value in register rY, store in rX
0xC1XY - Subtract value in register rX with value in register rY, store in rX
0xC2XY - Multiply value in register rX with value in register rY, store in rX
0xC3XY - Divide value in register rX with value in register rY, store in rX
0xC4XY - Find remainder of value in register rX with value in register rY, store in rX
0xC5XY - Bitwise AND value in register rX with value in register rY, store in rX
0xC6XY - Bitwise OR value in register rX with value in register rY, store in rX
0xC7XY - Bitwise XOR value in register rX with value in register rY, store in rX
0xC8XY - Bitwise NOT value in register rX, store in rX (rY can be anything, and will be ignored)
0xC9XY - Shift left value in register rX by value in register rY, store in rX
0xCAXY - Shift right value in register rX by value in register rY, store in rX
0xCBXY - Rotate left value in register rX by value in register rY, store in rX
0xCCXY - Rotate right value in register rX by value in register rY, store in rX
0xD00X - Store a random u16 into register rX
0xEKKK - Set index register to position KKK
0xF000 - Jump to address in following word
0xF100 - Call subroutine at address in following word
0xF200 - Set index register to address in following word
0xF30X - Set register rX to following word
0xFFFF - Halt
