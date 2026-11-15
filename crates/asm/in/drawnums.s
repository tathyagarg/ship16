START:
  SETI 0
  CLS
  MOV  r0, 0  ; num counter
  MOV  r1, 0  ; xpos counter
  MOV  r2, 24 ; limit

RLOOP:
  DRWSR r0
  SKP.NE r1, 7
  INCY 8
  INCX 8
  SKP.NE r0, r2
  JMP END
  ADD r0, 1
  ADD r1, 1
  REM r1, 8
  JMP RLOOP

END:
  KSKP 0
  JMP CHECK_2
  MOV r2, 0
  JMP RESTART

CHECK_2:
  KSKP 1
  JMP CHECK_3
  MOV r2, 1
  JMP RESTART

CHECK_3:
  KSKP 2
  JMP END
  MOV r2, 24
  JMP RESTART

RESTART:
  CLS
  MOV r0, 0
  MOV r1, 0
  MOVC 0, 0
  JMP RLOOP
