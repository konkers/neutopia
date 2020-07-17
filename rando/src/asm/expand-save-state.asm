arch pce.cpu

macro seek(variable offset) {
  origin (offset - $0)
  base offset
}

// bypass second an third rng pulls for salt
seek($d979);
    cla
    nop
    nop

seek($d9bb);
    cla
    nop
    nop


// patch checksum calc and verify to operate on whole 24 bytes

// these two patch the calc portion
seek($da53);
    lda #$16

seek($da6e);
    lda #$15

// these two patch the verify portion
seek($d8a5);
    ora #$15
    tax
    lda #$15

seek($d8bb);
    lda #$16


// patch salt funtion to salt whole buffer
seek($da88);
    lda #$16

// patch decode to only call salt and verify functions once
seek($d778);
    cla
    jsr $d8a2
    bcc $d796
    bra $d798

// patch encode to only call salt and verify functions once
seek($da03);
    cla
    jsr $da81
    bra $da1b
