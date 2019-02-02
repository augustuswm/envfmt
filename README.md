# envfmt

A small command line utility for reading parameters from a path in
the AWS Systems Manager Parameter Store and outputting them in a given
format.

Parameters are expected to have keys stored in Parameter Store under an
AWS path format.

`/path1/path2/path3/param`

Two output formats are currently support: `.env` and `php-fpm.conf`

`envfmt /path/to/ dot-env > .env`

`envfmt /path/to/ php-fpm > env.conf`

The region to use can be specified with the `region` flag.

`envfmt /path/to/ dot-env --region us-west-1 > .env`

If left unspecified the region will attempt to be read from the current
environment. In the case that it fails, it will fall back to us-east-1.
