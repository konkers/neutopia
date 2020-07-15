arch pce.cpu

macro seek(variable offset) {
  origin (offset - $6000)
  base offset
}

seek($ddf1);
get_sword_handler:
    dw handle_upgradeable_item 
    dw handle_upgradeable_item
    dw handle_upgradeable_item

seek($df52);
handle_upgradeable_item:
    ldx  $35d0 // chest item id
    lda  $35d1 // chest arg

    // Only update and item if it is better than the current one.
    cmp $2e44,x
    bcc skip_write
    sta $2e44,x
skip_write:

    // Store the value back to $35d1 so the rest of the code knows
    // which item to show.
    lda  $2e44,x
    sta  $35d1

    jmp  $de79
