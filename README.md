# UTC Engineering Clock

`eng-clock` is a simple Rust application that shows a live display of the current time,
but taking particular care to accurately synchronize screen updates to transitions
from one second to the next. (Note that some desktop clock applications may
simply provide updates every second, but at an arbitrary offset from
these second boundaries.)

The application sends its own NTP requests to feed into a statistical estimator
of the local system clock's offset from authoritative time references,
including an estimate of the accumulated margin of error.
(That offset estimator is based on a simple Bayesian Inference process
assuming Gaussian statistics.)

The visual display is deliberately minimalistic, using only basic GTK elements.


## Licensing

All files are released under the
[GPL-v3](https://www.gnu.org/licenses/gpl-3.0.en.html)
and are Copyright (C) 2023 RW Penney.
