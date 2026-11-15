START:
  SETI 0
  MOV  r0, 0  ; num counter
  MOV  r1, 0  ; xpos counter

RLOOP:
  DRWSR r0
  SKP.NE r1, 7
  INCY 8
  INCX 8
  SKP.NE r0, 24
  JMP END
  ADD r0, 1
  ADD r1, 1
  REM r1, 8
  JMP RLOOP

END:
  JMP END
