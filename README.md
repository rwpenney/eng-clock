# UTC Engineering Clock

`eng-clock` is a simple Rust application that shows a live display of the current time,
but taking particular care to accurately synchronize screen updates to transitions
from one second to the next. (Note that some desktop clock applications may
simply provide updates every second, but at an arbitrary offset from
these second boundaries.)

The application currently assumes that the system clock is itself well
synchonized to an external time reference (e.g. via NTP or GPS),
but may eventually have functionality that will compute a statistical
estimate of this local clock offset.

The visual display is deliberately minimalistic, using only basic GTK elements.


## Licensing

All files are released under the
[GPL-v3](https://www.gnu.org/licenses/gpl-3.0.en.html)
and are Copyright (C) 2023 RW Penney.
