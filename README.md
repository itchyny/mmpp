# Mackerel metric pretty printer
### Pretty printer cli for customized graph of Mackerel

Ref: https://mackerel.io/docs/entry/advanced/advanced-graph

```sh
 $ echo 'diff(service(Blog, foo.bar), service(Blog, foo.baz))' | mmpp
diff(
  service(Blog, foo.bar),
  service(Blog, foo.baz)
)
 $ echo 'avg(group(host(22CXRB3pZmu, loadavg5), host(22CXRB3pZmu, loadavg5)))' | mmpp
avg(
  group(
    host(22CXRB3pZmu, loadavg5),
    host(22CXRB3pZmu, loadavg5)
  )
)
 $ mmpp <<EOF
> alias(scale(timeLeftForecast(host('22CXRB3pZmu', 'filesystem.drive.used'), '3mo', 2000000000000), 1/86400), 'linear regresson sample')
> EOF
alias(
  scale(
    timeLeftForecast(
      host(22CXRB3pZmu, filesystem.drive.used),
      3mo,
      2000000000000
    ),
    1/86400
  ),
  'linear regresson sample'
)
```

## Author
itchyny (https://github.com/itchyny)

## License
This software is released under the MIT License, see LICENSE.
