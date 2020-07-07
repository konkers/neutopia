arch pce.cpu

origin $88f1;
    dec $FA
    bne done
    lda #$01
    sta $FA
    jsr $896A
done:
    rts