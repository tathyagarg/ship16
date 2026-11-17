.section text

; ==========================================
; CONWAY'S GAME OF LIFE (toroidal)
;
; The 64x64 world is stored in two padded 66x66 grids, so each cell's
; eight neighbours are fixed offsets and no per-neighbour wrapping is
; needed. The one-cell ring around the world mirrors the opposite edge
; and is refreshed once per generation.
;
; Cell (x, y) lives at  base + (y + 1) * 66 + (x + 1).
;
;   0xC000  display framebuffer (64x64): 0x00 dead, 0xFF alive, 0x7F cursor
;   0xD000  current generation, padded 66x66 (0 dead, 1 alive)  -> 0xE103
;   0xE200  next generation,    padded 66x66 (0 dead, 1 alive)  -> 0xF303
;
; rE = number of completed generations (wraps at 65536)
;
; Editor keys: 1 left, 2 right, 0 up, 9 down, F toggle cell, A start.
; ==========================================


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

  ; Clear current generation (4356 = 66 * 66)
  SETI 0xD000
  MOV r0, 4356

CLEAR_GRID:
  ST8 r1
  AI 1
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP CLEAR_GRID

  ; Clear next-generation buffer
  SETI 0xE200
  MOV r0, 4356

CLEAR_NEXT:
  ST8 r1
  AI 1
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP CLEAR_NEXT

  ; Initialize cursor
  MOV r0, 32
  MOV r1, 32
  MOV rD, 0
  MOV rE, 0                ; generation counter
  JMP DRAW_CURSOR


; ==========================================
; EDITOR
;
; r0 = cursor X (0..63)
; r1 = cursor Y (0..63)
; r5 = display offset  y * 64 + x
; r6 = grid offset     y * 66 + x + 67
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
; TOGGLE CELL
;
; F flips the cell under the cursor, once per key press.
; rD = 1 while F is being held, so holding the key does not
; flip the cell every frame.
; ==========================================

CHECK_PLACE:
  KSKP 15
  JMP PLACE_RELEASED

  SKP<EQ> rD, 0
  JMP EDITOR               ; still held from an earlier frame

  MOV rD, 1

  MOV r6, r1
  MUL r6, 66
  ADD r6, r0
  ADD r6, 67

  ; Flip the authoritative grid cell (0 <-> 1)
  SETI 0xD000
  AI r6
  LD8 r8
  XOR r8, 1
  ST8 r8

  ; Cursor remains drawn over the cell
  JMP EDITOR

PLACE_RELEASED:
  MOV rD, 0
  JMP CHECK_START


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

  MOV r6, r1
  MUL r6, 66
  ADD r6, r0
  ADD r6, 67

  SETI 0xD000
  AI r6
  LD8 r8
  MUL r8, 255

  SETI 0xC000
  AI r5
  ST8 r8

  JMP UPDATE


; ==========================================
; MOVE CURSOR
; ==========================================

MOVE_CURSOR:
  ; Restore previous cursor position from the grid
  MOV r5, r1
  MUL r5, 64
  ADD r5, r0

  MOV r6, r1
  MUL r6, 66
  ADD r6, r0
  ADD r6, 67

  SETI 0xD000
  AI r6
  LD8 r8
  MUL r8, 255

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
  ADD r1, 1
  SKP<EQ> r1, 64
  JMP DRAW_CURSOR

  MOV r1, 0
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

; ------------------------------------------
; Refresh the ghost ring of the current grid
; (padded rows/columns 0 and 65 mirror the opposite edge)
; ------------------------------------------

  ; Row 0, columns 1..64  <-  row 64, columns 1..64
  MOV r5, 0xE081
  SETI 0xD001
  MEMCPY r5, 64

  ; Row 65, columns 1..64  <-  row 1, columns 1..64
  MOV r5, 0xD043
  SETI 0xE0C3
  MEMCPY r5, 64

  ; Columns 0 and 65 of all 66 rows (also fills the corners)
  MOV r6, 0
  MOV r0, 66

GHOST_COLUMNS:
  SETI 0xD040              ; column 64 -> column 0
  AI r6
  LD8 r8
  SETI 0xD000
  AI r6
  ST8 r8

  SETI 0xD001              ; column 1 -> column 65
  AI r6
  LD8 r8
  SETI 0xD041
  AI r6
  ST8 r8

  ADD r6, 66
  SUB r0, 1
  SKP<EQ> r0, 0
  JMP GHOST_COLUMNS

; ------------------------------------------
; Compute the next generation
;
; Neighbour offsets from a cell:
;
;   -67 -66 -65
;    -1   .  +1
;   +65 +66 +67
;
; AI with a 16-bit immediate wraps, so AI 0xFFBD subtracts 67.
;
; r0 = X                 r4 = next cell (0 or 1)
; r1 = Y                 r5 = display offset  y * 64 + x
; r2 = neighbour count   r6 = grid offset     y * 66 + x + 67
; r3 = current cell      r8 = temporary
;
; A cell is alive next generation when (neighbours OR current) == 3.
; ------------------------------------------

  MOV r0, 0
  MOV r1, 0
  MOV r5, 0
  MOV r6, 67

CELL:
  SETI 0xD000
  AI r6
  LD8 r3
  MOV r2, 0

  AI 0xFFBD                ; -67
  LD8 r8
  ADD r2, r8
  AI 1                     ; -66
  LD8 r8
  ADD r2, r8
  AI 1                     ; -65
  LD8 r8
  ADD r2, r8
  AI 64                    ; -1
  LD8 r8
  ADD r2, r8
  AI 2                     ; +1
  LD8 r8
  ADD r2, r8
  AI 64                    ; +65
  LD8 r8
  ADD r2, r8
  AI 1                     ; +66
  LD8 r8
  ADD r2, r8
  AI 1                     ; +67
  LD8 r8
  ADD r2, r8

  OR r2, r3
  MOV r4, 0
  SKP<NE> r2, 3
  MOV r4, 1

  ; Store into next-generation buffer
  SETI 0xE200
  AI r6
  ST8 r4

  ; Update display (0 -> 0x00, 1 -> 0xFF)
  MOV r8, r4
  MUL r8, 255
  SETI 0xC000
  AI r5
  ST8 r8

  ADD r5, 1
  ADD r6, 1
  ADD r0, 1
  SKP<EQ> r0, 64
  JMP CELL

  ; End of row: skip the two ghost columns
  MOV r0, 0
  ADD r6, 2
  ADD r1, 1
  SKP<EQ> r1, 64
  JMP CELL


; ==========================================
; COPY NEXT GENERATION
; 0xE200 -> 0xD000 (the display was updated per cell)
; ==========================================

  MOV r5, 0xE200
  SETI 0xD000
  MEMCPY r5, 4356

  ADD rE, 1                ; one more generation completed
  JMP UPDATE
