arch pce.cpu

macro seek(variable offset) {
  origin (offset - $6000)
  base offset
}

seek($ddf1);
get_sword_handler:
    dw handle_progressive_item 
    dw handle_progressive_item
    dw handle_progressive_item

seek($df52);
handle_progressive_item:
    ldx  $35d0
    inc  $2e44,x

    // Store the value back to $35d1 so the rest of the code knows
    // which item to show.
    lda  $2e44,x
    sta  $35d1

    jmp  $de79
