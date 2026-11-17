.section text

; ==========================================
; INITIALIZATION
; ==========================================

START:
  MOVC 0, 0
  CLS

  TCFG 1, 1
  MOV r4, 5
  TC r4
  TR r4
  ENINT 0b010
  EI

  ; Clear display framebuffer
  SETI 0xC000
  MOV r0, 4096
  MOV r1, 0

CLEAR_DISPLAY:
  ST8 r1
  AI 1
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP CLEAR_DISPLAY

  ; Clear authoritative grid
  SETI 0xD000
  MOV r0, 4096

CLEAR_GRID:
  ST8 r1
  AI 1
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP CLEAR_GRID

  ; Clear next-generation buffer
  SETI 0xE000
  MOV r0, 4096

CLEAR_NEXT:
  ST8 r1
  AI 1
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP CLEAR_NEXT

  ; Initialize cursor
  MOV r0, 32
  MOV r1, 32
  JMP DRAW_CURSOR


; ==========================================
; EDITOR
;
; r0 = cursor X
; r1 = cursor Y
; rC = movement direction
;
; 0 = left
; 1 = right
; 2 = up
; 3 = down
; ==========================================

EDITOR:
  WAIT

  KSKP 1
  JMP CHECK_RIGHT

  MOV rC, 0
  JMP MOVE_CURSOR

CHECK_RIGHT:
  KSKP 2
  JMP CHECK_UP

  MOV rC, 1
  JMP MOVE_CURSOR

CHECK_UP:
  KSKP 0
  JMP CHECK_DOWN

  MOV rC, 2
  JMP MOVE_CURSOR

CHECK_DOWN:
  KSKP 9
  JMP CHECK_PLACE

  MOV rC, 3
  JMP MOVE_CURSOR


; ==========================================
; PLACE CELL
; ==========================================

CHECK_PLACE:
  KSKP 15
  JMP CHECK_START

  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  ; Update authoritative grid
  SETI 0xD000
  AI r5
  MOV r8, 0xFF
  ST8 r8

  ; Cursor remains drawn over the cell
  JMP EDITOR


; ==========================================
; START SIMULATION
; ==========================================

CHECK_START:
  KSKP 10
  JMP EDITOR

  ; Restore the cell underneath the cursor
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  SETI 0xD000
  AI r5
  LD8 r8

  SETI 0xC000
  AI r5
  ST8 r8

  JMP UPDATE


; ==========================================
; MOVE CURSOR
; ==========================================

MOVE_CURSOR:
  ; Restore previous cursor position
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  SETI 0xD000
  AI r5
  LD8 r8

  SETI 0xC000
  AI r5
  ST8 r8

  ; LEFT
  SKP<EQ> rC, 0
  JMP MOVE_RIGHT

  SKP<EQ> r0, 0
  JMP LEFT_NORMAL

  MOV r0, 63
  JMP DRAW_CURSOR

LEFT_NORMAL:
  SUB r0, 1
  JMP DRAW_CURSOR

  ; RIGHT
MOVE_RIGHT:
  SKP<EQ> rC, 1
  JMP MOVE_UP

  ADD r0, 1
  SKP<EQ> r0, 64
  JMP DRAW_CURSOR

  MOV r0, 0
  JMP DRAW_CURSOR

  ; UP
MOVE_UP:
  SKP<EQ> rC, 2
  JMP MOVE_DOWN

  SKP<EQ> r1, 0
  JMP UP_NORMAL

  MOV r1, 63
  JMP DRAW_CURSOR

UP_NORMAL:
  SUB r1, 1
  JMP DRAW_CURSOR

  ; DOWN
MOVE_DOWN:
  SKP<EQ> r1, 63
  JMP DOWN_NORMAL

  MOV r1, 0
  JMP DRAW_CURSOR

DOWN_NORMAL:
  ADD r1, 1
  JMP DRAW_CURSOR


; ==========================================
; DRAW CURSOR
; ==========================================

DRAW_CURSOR:
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  SETI 0xC000
  AI r5
  MOV r8, 0x7F
  ST8 r8

  JMP EDITOR


; ==========================================
; GAME OF LIFE
; ==========================================

UPDATE:
  MOV r0, 0
  MOV r1, 0

; r0 = X
; r1 = Y
; r2 = neighbour count
; r3 = current cell
; r4 = next cell
; r5 = memory offset
; r6 = neighbour X
; r7 = neighbour Y
; r8 = temporary value
; r9 = X direction
; rA = Y direction

CELL_LOOP:
  ; Read current cell
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  SETI 0xD000
  AI r5
  LD8 r3

  MOV r2, 0

  ; Neighbour 1: (-1, -1)
  MOV r9, 0
  MOV rA, 0
  CALL COUNT_AT

  ; Neighbour 2: (0, -1)
  MOV r9, 1
  MOV rA, 0
  CALL COUNT_AT

  ; Neighbour 3: (+1, -1)
  MOV r9, 2
  MOV rA, 0
  CALL COUNT_AT

  ; Neighbour 4: (-1, 0)
  MOV r9, 0
  MOV rA, 1
  CALL COUNT_AT

  ; Neighbour 5: (+1, 0)
  MOV r9, 2
  MOV rA, 1
  CALL COUNT_AT

  ; Neighbour 6: (-1, +1)
  MOV r9, 0
  MOV rA, 2
  CALL COUNT_AT

  ; Neighbour 7: (0, +1)
  MOV r9, 1
  MOV rA, 2
  CALL COUNT_AT

  ; Neighbour 8: (+1, +1)
  MOV r9, 2
  MOV rA, 2
  CALL COUNT_AT

  ; Apply Conway's rules
  MOV r4, 0

  ; Exactly three neighbours: alive
  SKP<EQ> r2, 3
  JMP CHECK_TWO

  MOV r4, 0xFF
  JMP STORE_NEXT

CHECK_TWO:
  ; Exactly two neighbours: preserve state
  SKP<EQ> r2, 2
  JMP STORE_NEXT

  MOV r4, r3

STORE_NEXT:
  ; Store into next-generation buffer
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  SETI 0xE000
  AI r5
  ST8 r4

  ; Advance X
  ADD r0, 1

  SKP<EQ> r0, 64
  JMP CELL_LOOP

  MOV r0, 0
  ADD r1, 1

  SKP<EQ> r1, 64
  JMP CELL_LOOP


; ==========================================
; COPY NEXT GENERATION
; E000 -> D000 and C000
; ==========================================

  MOV r0, 0

COPY_LOOP:
  MOV r5, 0xE000
  SETI 0xD000
  MEMCPY r5, 4096

  MOV r5, 0xD000
  SETI 0xC000
  MEMCPY r5, 4096

  JMP UPDATE

  ; SETI 0xE000
  ; AI r0
  ; LD8 r8

  ; ; Update authoritative grid
  ; SETI 0xD000
  ; AI r0
  ; ST8 r8

  ; ; Update display
  ; SETI 0xC000
  ; AI r0
  ; ST8 r8

  ; ADD r0, 1

  ; SKP<EQ> r0, 4096
  ; JMP COPY_LOOP

  ; JMP UPDATE


; ==========================================
; COUNT_AT
;
; r9 = X direction
; rA = Y direction
;
; 0 = decrement
; 1 = unchanged
; 2 = increment
;
; Increments r2 if neighbour is alive.
; ==========================================

COUNT_AT:
  MOV r6, r0
  MOV r7, r1

  ; Wrap X coordinate

  SKP<EQ> r9, 0
  JMP CX_INCREMENT

  SKP<EQ> r6, 0
  JMP CX_DECREMENT

  MOV r6, 63
  JMP CY_START

CX_DECREMENT:
  SUB r6, 1
  JMP CY_START

CX_INCREMENT:
  SKP<EQ> r9, 2
  JMP CY_START

  ADD r6, 1
  SKP<EQ> r6, 64
  JMP CY_START

  MOV r6, 0


  ; Wrap Y coordinate

CY_START:
  SKP<EQ> rA, 0
  JMP CY_INCREMENT

  SKP<EQ> r7, 0
  JMP CY_DECREMENT

  MOV r7, 63
  JMP READ_NEIGHBOUR

CY_DECREMENT:
  SUB r7, 1
  JMP READ_NEIGHBOUR

CY_INCREMENT:
  SKP<EQ> rA, 2
  JMP READ_NEIGHBOUR

  ADD r7, 1
  SKP<EQ> r7, 64
  JMP READ_NEIGHBOUR

  MOV r7, 0


READ_NEIGHBOUR:
  ; Calculate address
  MOV r5, r7
  MUL r5, 64
  ADD r5, r6

  SETI 0xD000
  AI r5
  LD8 r8

  ; Increment neighbour count if alive
  SKP<NE> r8, 0
  JMP COUNT_DONE

  ADD r2, 1

COUNT_DONE:
  RET

