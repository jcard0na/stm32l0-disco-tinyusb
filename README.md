# Build

```
cargo make build
```

# Flash

1. Insert USB cable on USB-ST-LINK port of discovery board
2. Flash
```
cargo make flash
```

# Debug 

1. Add hprint! or hprintln! statements to the code
2. Flash with semihosting enabled
```
cargo make semiflash
```
3. Debug messages should appear on the terminal
