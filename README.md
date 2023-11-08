# high-voltage-line-distance
A cli tool to calculate distance to the nearest high voltage line in Sydney

# Usage
```
[prog] -a <address> [-v] [--no-cache]
```
for example:
```
[prog] -a "56 Iris Street Frenchs Forest, NSW"
```
and it will print
> 06:02:02 [INFO] 1884m away from 330kV power line
> 
> 06:02:02 [INFO] 548m away from 132kV power line

## Argument
### [required] address
`-a` or `--address`. The address to the location that you want to know how far it is to the high voltage power line. Just pass whatever you will search on google map.
It should be noted that if there are multiple address found based on the address given, the first one will be used.

### [optional] verbose
`-v`: Print debug messages, default to `false`

### [optional] no cache
`--no-cache`: By default, api calls are cached for 24 hours because they don't got changed often. Default to `false`
