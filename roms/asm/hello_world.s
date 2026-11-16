.section text

START:
  PUSH   37
  PUSH   6
  PUSH   1
  PUSH   25
  PUSH   18
  PUSH   17
  PUSH   28
  PUSH   36
  PUSH   24
  PUSH   21
  PUSH   21
  PUSH   14
  PUSH   17

  MOVC   0, 0
  CLS
  MOV    r0, 0
  MOV    r1, 13
  MOV    rF, 56

  TCFG 1, 1
  MOV  r4, 30
  TC   r4
  TR   r4
  ENINT 2

DRAW:
  POP    r2
  DRWSR  r2
  SKP<NE> r0, 5
  CALL   NEXTLINE
  INCX   8
  ADD    r0, 1
  SUB    r1, 1
  WAIT
  FSKP   Z
  JMP    DRAW
  JMP    END

NEXTLINE:
  INCY   8
  SETX   rF
  RET

END:
  JMP END
