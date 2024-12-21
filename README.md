# trashgb
Yet another Gameboy emulator the world doesn't need.

A personal project to learn more about emulation, Rust, and targetting
WebAssembly. While largely compatible with many commercial games, there are
many other emulators out there I would recommend instead if you are just
looking to play Gameboy games.

I do not condone software piracy. Please only use this emulator with software which you own or have license to.

### Screenshots
![Kirby](screenshots/kirby.png)
![Super Mario Land](screenshots/marioland.png)
![Metroid II](screenshots/metroid2.png)

### Usage
#### Linux/MacOS
```sh
./trashgb <rom_file>
```

#### Windows
```sh
trashgb.exe <rom_file>
```

#### Web
Visit [syph.io/gb](https://syph.io/gb)

### Key Bindings
| Key         | Action |
| ----------- | ------ |
| `↑`         | Up     |
| `↓`         | Down   |
| `←`         | Left   |
| `→`         | Right  |
| `Z`         | A      |
| `X`         | B      |
| `Enter`     | Start  |
| `Backspace` | Select |

### Gameboy Test ROMs

#### [Blarrg's Gameboy hardware test ROMs](https://github.com/retrio/gb-test-roms)
**Instruction tests**:

![Blargg's Instruction Tests](screenshots/cpu_instrs.png)
![Blargg's Instruction Timing Tests](screenshots/instr_timing.png)
 - [X] cpu_instrs\.gb
 - [X] instr_timing\.gb

#### [Mooneye Test Suite](https://github.com/Gekkio/mooneye-test-suite)
**acceptance/**:
 - [X] bits/mem_oam
 - [X] bits/reg_f
 - [X] instr/daa
 - [X] oam_dma/basic
 - [ ] oam_dma/reg_read
 - [X] timer/div_write
 - [ ] timer/rapid_toggle
 - [X] timer/tim00
 - [ ] timer/tim00_div_trigger
 - [X] timer/tim01
 - [ ] timer/tim01_div_trigger
 - [X] timer/tim11
 - [ ] timer/tim11_div_trigger
 - [ ] timer/tima_reload
 - [ ] timer/tma_write_reloading

**emulator-only/mbc1/**:
 - [X] bits\_bank1
 - [X] bits\_bank2
 - [X] bits\_mode
 - [X] bits\_ramg
 - [ ] multicart\_rom\_8Mb
 - [X] ram\_256kb
 - [X] ram\_64kb
 - [X] rom\_16Mb
 - [X] rom\_1Mb
 - [X] rom\_2Mb
 - [X] rom\_4Mb
 - [X] rom\_512kb
 - [X] rom\_8Mb

**manual-only/**:
 - [X] sprite_priority

#### Other tests:

![dmg-acid2](screenshots/dmg-acid2.png)
 - [X] [dmg-acid2](https://github.com/mattcurrie/dmg-acid2)

### Acknowledgements
This emulator makes use of Hacktix's open source boot ROM, [Bootix](https://github.com/Hacktix/Bootix).

### License
This project is licensed under the MIT License - see the [LICENSE](LICENSE)
file for details.
