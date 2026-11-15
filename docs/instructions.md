# Instructions

0x0000 - NOP - No op
0x020X - LD8 r<0-F> - Load 8 bits from location pointed by index register into register rX
0x021X - ST8 r<0-F> - Store 8 bits from register rX into location pointed by index register
0x022X - LD16 r<0-F> - Load 16 bits from location pointed by index register into register rX
0x023X - ST16 r<0-F> - Store 16 bits from register rX into location pointed by index register
0x024X - AI r<0-F> - Move index register forward by value held in register rX
0x030X - PUSH r<0-F> - Push value held in register rX onto the stack
0x031X - POP r<0-F> - Pop value from the stack into register rX
0x040k - FNSKP [Z|C|O|N] - Skip next instruction if flag number k is set to OFF
0x041k - FSKP [Z|C|O|N] - Skip next instruction if flag number k is set to ON
0x1000 - CLS - Clear the display
0x110X - TC r<0-F> - Set timer counter to value held in register rX
0x111X - TR r<0-F> - Set timer reload to value held in register rX
0x112X - LDTC r<0-F> - Load timer counter into register rX
0x113k - TCFG [0|1], [0|1] - Set timer configuration to k (bit 0: periodic, bit 1: enabled)
0x120k - ENINT \d - Enable interrupt k
0x121k - DISINT \d - Disable interrupt k
0x130k - VEC \d - Set interrupt vector for interrupt k to value held in index register
0x1400 - EI - Enable interrupts
0x1401 - DI - Disable interrupts
0x1500 - WAIT - Wait
0x20KK - DRW \d - Draw color KK at cursor position
0x210X - DRWR r<0-F> - Draw color held in register rX at cursor position
0x22KK - DRWS \d - Draw sprite KK at cursor position
0x230X - DRWSR r<0-F> - Draw sprite held in register rX at cursor position
0x3KKK - MOVC \d, \d - Move cursor to position KKK (top 6 bits: X pos, bottom X bits: Y pos)
0x40KK - INCX \d - Increment cursor X by KK (wrapping to 0 if overflow)
0x41KK - INCY \d - Increment cursor Y by KK (wrapping to 0 if overflow)
0x42KK - DECX \d - Decrement cursor X by KK (wrapping to 0 if overflow)
0x43KK - DECY \d - Decrement cursor Y by KK (wrapping to 0 if overflow)
0x4A0X - SETX r<0-F> - Set cursor X to value held in register rX
0x4B0X - SETY r<0-F> - Set cursor Y to value held in register rX
0x5KKK - JMP \d - Jump to position KKK
0x60XY - SKP.EQ r<0-F>, r<0-F> - Skip next instruction if value in rX == value in rY
0x61XY - SKP.NE r<0-F>, r<0-F> - Skip next instruction if value in rX != value in rY
0x62XY - SKP.LT r<0-F>, r<0-F> - Skip next instruction if value in rX < value in rY
0x63XY - SKP.GT r<0-F>, r<0-F> - Skip next instruction if value in rX > value in rY
0x64XY - SKP.LE r<0-F>, r<0-F> - Skip next instruction if value in rX <= value in rY
0x65XY - SKP.GE r<0-F>, r<0-F> - Skip next instruction if value in rX >= value in rY
0x7XKK - MOV r<0-F>, \d - Move KK into register rX
0x80XY - MOV r<0-F>, r<0-F> - Copy value in register rX into register rY
0x9KKK - CALL \d - Call subroutine at address KKK
0xA000 - RET - Return from subroutine
0xA001 - RETI - Return from interrupt vector
0xB00K - KSKP \d - Skip next instruction if key K is pressed
0xC0XY - ADD r<0-F>, r<0-F> - Add value in register rX with value in register rY, store in rX
0xC1XY - SUB r<0-F>, r<0-F> - Subtract value in register rX with value in register rY, store in rX
0xC2XY - MUL r<0-F>, r<0-F> - Multiply value in register rX with value in register rY, store in rX
0xC3XY - DIV r<0-F>, r<0-F> - Divide value in register rX with value in register rY, store in rX
0xC4XY - REM r<0-F>, r<0-F> - Find remainder of value in register rX with value in register rY, store in rX
0xC5XY - AND r<0-F>, r<0-F> - Bitwise AND value in register rX with value in register rY, store in rX
0xC6XY - OR r<0-F>, r<0-F> - Bitwise OR value in register rX with value in register rY, store in rX
0xC7XY - XOR r<0-F>, r<0-F> - Bitwise XOR value in register rX with value in register rY, store in rX
0xC8XY - NOT r<0-F>, r<0-F> - Bitwise NOT value in register rX, store in rX (rY can be anything, and will be ignored)
0xC9XY - SHL r<0-F>, r<0-F> - Shift left value in register rX by value in register rY, store in rX
0xCAXY - SHR r<0-F>, r<0-F> - Shift right value in register rX by value in register rY, store in rX
0xCBXY - ROL r<0-F>, r<0-F> - Rotate left value in register rX by value in register rY, store in rX
0xCCXY - ROR r<0-F>, r<0-F> - Rotate right value in register rX by value in register rY, store in rX
0xD00X - RAND r<0-F> - Store a random u16 into register rX
0xEKKK - SETI r<0-F> - Set index register to position KKK
0xF000 - JMP \d - Jump to address in following word
0xF100 - CALL \d - Call subroutine at address in following word
0xF200 - SETI \d - Set index register to address in following word
0xF30X - MOV r<0-F>, \d - Set register rX to following word
0xF40X - ADD r<0-F>, \d - Add value in register rX with following word, store in rX
0xF41X - SUB r<0-F>, \d - Subtract value in register rX with following word, store in rX
0xF42X - MUL r<0-F>, \d - Multiply value in register rX with following word, store in rX
0xF43X - DIV r<0-F>, \d - Divide value in register rX with following word, store in rX
0xF44X - REM r<0-F>, \d - Find remainder of value in register rX with following word, store in rX
0xF45X - AND r<0-F>, \d - Bitwise AND value in register rX with following word, store in rX
0xF46X - OR r<0-F>, \d - Bitwise OR value in register rX with following word, store in rX
0xF47X - XOR r<0-F>, \d - Bitwise XOR value in register rX with following word, store in rX
0xF48X - NOT r<0-F>, \d - Bitwise NOT value in register rX, store in rX (word is ignored)
0xF49X - SHL r<0-F>, \d - Shift left value in register rX with following word, store in rX
0xF4AX - SHR r<0-F>, \d - Shift right value in register rX with following word, store in rX
0xF4BX - ROL r<0-F>, \d - Rotate left value in register rX with following word, store in rX
0xF4CX - ROR r<0-F>, \d - Rotate right value in register rX with following word, store in rX
0xF50X - SKP.EQ r<0-F>, \d - Skip next instruction if value in rX == following word
0xF51X - SKP.NE r<0-F>, \d - Skip next instruction if value in rX != following word
0xF52X - SKP.LT r<0-F>, \d - Skip next instruction if value in rX < following word
0xF53X - SKP.GT r<0-F>, \d - Skip next instruction if value in rX > following word
0xF54X - SKP.LE r<0-F>, \d - Skip next instruction if value in rX <= following word
0xF55X - SKP.GE r<0-F>, \d - Skip next instruction if value in rX >= following word
0xFFFF - HLT - Halt
