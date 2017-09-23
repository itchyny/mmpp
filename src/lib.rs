extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::Pair;
use pest::inputs::Input;

#[derive(Parser)]
#[grammar = "metrics.pest"]
pub struct MetricParser;

#[derive(Debug, PartialEq, Clone)]
pub enum Metric {
    Host(String, String),
    Service(String, String),
    Role(String, String, String),
    RoleSlot(String, String, String),
    Avg(Box<Metric>),
    Max(Box<Metric>),
    Min(Box<Metric>),
    Product(Box<Metric>),
    Diff(Box<Metric>, Box<Metric>),
    Divide(Box<Metric>, Box<Metric>),
    Scale(Box<Metric>, Factor),
    Offset(Box<Metric>, Factor),
    Percentile(Box<Metric>, Percentage),
    TimeShift(Box<Metric>, Duration),
    MovingAverage(Box<Metric>, Duration),
    LinearRegression(Box<Metric>, Duration),
    TimeLeftForecast(Box<Metric>, Duration, Factor),
    Group(Vec<Metric>),
    Stack(Box<Metric>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Factor {
    Double(String),
    Fraction(String, String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Percentage(String);

#[derive(Debug, PartialEq, Clone)]
pub struct Duration(String);

macro_rules! next {
    ($pairs:expr) => {
        $pairs.next().unwrap()
    }
}

macro_rules! arg {
    ($pairs:expr) => {
        next!(next!($pairs).into_inner())
    }
}

macro_rules! arg_str {
    ($pairs:expr) => {
        arg!($pairs).as_str().to_string()
    }
}

pub fn parse_metric(src: &str) -> Result<Metric, String> {
    let mut pairs = MetricParser::parse_str(Rule::whole_metrics, src).map_err(|e| format!("{}", e))?;
    convert_metrics(next!(next!(pairs).into_inner()))
}

fn convert_metrics<I: Input>(pair: Pair<Rule, I>) -> Result<Metric, String> {
    match pair.as_rule() {
        Rule::host_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Host(arg_str!(inner), arg_str!(inner)))
        }
        Rule::service_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Service(arg_str!(inner), arg_str!(inner)))
        }
        Rule::role_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = arg!(inner).into_inner();
            let service_name = next!(role_full_name).as_str().to_string();
            let role_name = next!(role_full_name).as_str().to_string();
            Ok(Metric::Role(service_name, role_name, arg_str!(inner)))
        }
        Rule::role_slot_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = arg!(inner).into_inner();
            let service_name = next!(role_full_name).as_str().to_string();
            let role_name = next!(role_full_name).as_str().to_string();
            Ok(Metric::RoleSlot(service_name, role_name, arg_str!(inner)))
        }
        Rule::avg_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Avg(Box::new(convert_metrics(next!(inner))?)))
        }
        Rule::max_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Max(Box::new(convert_metrics(next!(inner))?)))
        }
        Rule::min_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Min(Box::new(convert_metrics(next!(inner))?)))
        }
        Rule::product_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Product(Box::new(convert_metrics(next!(inner))?)))
        }
        Rule::diff_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Diff(
                Box::new(convert_metrics(next!(inner))?),
                Box::new(convert_metrics(next!(inner))?),
            ))
        }
        Rule::divide_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Divide(
                Box::new(convert_metrics(next!(inner))?),
                Box::new(convert_metrics(next!(inner))?),
            ))
        }
        Rule::scale_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Scale(Box::new(convert_metrics(next!(inner))?), convert_factor(arg!(inner))?))
        }
        Rule::offset_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Offset(Box::new(convert_metrics(next!(inner))?), convert_factor(arg!(inner))?))
        }
        Rule::percentile_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Percentile(
                Box::new(convert_metrics(next!(inner))?),
                convert_percentage(next!(inner))?,
            ))
        }
        Rule::time_shift_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::TimeShift(
                Box::new(convert_metrics(next!(inner))?),
                convert_duration(arg!(inner))?,
            ))
        }
        Rule::moving_average_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::MovingAverage(
                Box::new(convert_metrics(next!(inner))?),
                convert_duration(arg!(inner))?,
            ))
        }
        Rule::linear_regression_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::LinearRegression(
                Box::new(convert_metrics(next!(inner))?),
                convert_duration(arg!(inner))?,
            ))
        }
        Rule::time_left_forecast_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::TimeLeftForecast(
                Box::new(convert_metrics(next!(inner))?),
                convert_duration(arg!(inner))?,
                convert_factor(arg!(inner))?,
            ))
        }
        Rule::group_metric => {
            let mut metrics = Vec::new();
            for r in pair.into_inner() {
                metrics.push(convert_metrics(r)?);
            }
            Ok(Metric::Group(metrics))
        }
        Rule::stack_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::Stack(Box::new(convert_metrics(next!(inner))?)))
        }
        Rule::metrics => convert_metrics(next!(pair.into_inner())),
        _ => unreachable!(),
    }
}

fn convert_factor<I: Input>(pair: Pair<Rule, I>) -> Result<Factor, String> {
    match pair.as_rule() {
        Rule::double => Ok(Factor::Double(pair.as_str().to_string())),
        Rule::fraction => {
            let mut inner = pair.into_inner();
            Ok(Factor::Fraction(next!(inner).as_str().to_string(), next!(inner).as_str().to_string()))
        }
        r => Err(format!("invalid factor: {:?}", r)),
    }
}

fn convert_percentage<I: Input>(pair: Pair<Rule, I>) -> Result<Percentage, String> {
    match pair.as_rule() {
        Rule::double => Ok(Percentage(pair.as_str().to_string())),
        r => Err(format!("invalid percentage: {:?}", r)),
    }
}

fn convert_duration<I: Input>(pair: Pair<Rule, I>) -> Result<Duration, String> {
    match pair.as_rule() {
        Rule::duration => Ok(Duration(pair.as_str().to_string())),
        r => Err(format!("invalid percentage: {:?}", r)),
    }
}

pub fn pretty_print(metric: Metric) -> String {
    pretty_print_inner(metric.clone(), calc_depth(metric), 0)
}

fn calc_depth(metric: Metric) -> u64 {
    match metric {
        Metric::Avg(metric) => 1 + calc_depth(*metric),
        Metric::Max(metric) => 1 + calc_depth(*metric),
        Metric::Min(metric) => 1 + calc_depth(*metric),
        Metric::Product(metric) => 1 + calc_depth(*metric),
        Metric::Diff(metric1, metric2) => 1 + vec![calc_depth(*metric1), calc_depth(*metric2)].iter().max().unwrap(),
        Metric::Divide(metric1, metric2) => 1 + vec![calc_depth(*metric1), calc_depth(*metric2)].iter().max().unwrap(),
        Metric::Scale(metric, _) => 1 + calc_depth(*metric),
        Metric::Offset(metric, _) => 1 + calc_depth(*metric),
        Metric::Percentile(metric, _) => 1 + calc_depth(*metric),
        Metric::TimeShift(metric, _) => 1 + calc_depth(*metric),
        Metric::MovingAverage(metric, _) => 1 + calc_depth(*metric),
        Metric::LinearRegression(metric, _) => 1 + calc_depth(*metric),
        Metric::TimeLeftForecast(metric, _, _) => 1 + calc_depth(*metric),
        Metric::Group(metrics) => 1 + metrics.iter().map(|metric| calc_depth(metric.clone())).max().unwrap(),
        Metric::Stack(metric) => 1 + calc_depth(*metric),
        _ => 1,
    }
}

fn pretty_print_inner(metric: Metric, depth: u64, indent: usize) -> String {
    let indent_str = " ".repeat(indent * 2);
    let metric_str = match metric {
        Metric::Host(host_id, metric_name) => format!("host({}, {})", host_id, metric_name),
        Metric::Service(service_name, metric_name) => format!("service({}, {})", service_name, metric_name),
        Metric::Role(service_name, role_name, metric_name) => format!("role({}:{}, {})", service_name, role_name, metric_name),
        Metric::RoleSlot(service_name, role_name, metric_name) => format!("roleSlots({}:{}, {})", service_name, role_name, metric_name),
        Metric::Avg(metric) => if depth <= 2 {
            format!("avg({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("avg(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::Max(metric) => if depth <= 2 {
            format!("max({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("max(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::Min(metric) => if depth <= 2 {
            format!("min({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("min(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::Product(metric) => if depth <= 2 {
            format!("product({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("product(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::Diff(metric1, metric2) => format!(
            "diff(\n{},\n{}\n{})",
            pretty_print_inner(*metric1, depth - 1, indent + 1),
            pretty_print_inner(*metric2, depth - 1, indent + 1),
            indent_str
        ),
        Metric::Divide(metric1, metric2) => format!(
            "divide(\n{},\n{}\n{})",
            pretty_print_inner(*metric1, depth - 1, indent + 1),
            pretty_print_inner(*metric2, depth - 1, indent + 1),
            indent_str
        ),
        Metric::Scale(metric, factor) => if depth <= 2 {
            format!("scale({}, {})", pretty_print_inner(*metric, depth - 1, 0), pretty_print_factor(factor))
        } else {
            format!(
                "scale(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_factor(factor),
                indent_str
            )
        },
        Metric::Offset(metric, factor) => if depth <= 2 {
            format!("offset({}, {})", pretty_print_inner(*metric, depth - 1, 0), pretty_print_factor(factor))
        } else {
            format!(
                "offset(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_factor(factor),
                indent_str
            )
        },
        Metric::Percentile(metric, percentage) => if depth <= 2 {
            format!(
                "percentile({}, {})",
                pretty_print_inner(*metric, depth - 1, 0),
                pretty_print_percentage(percentage)
            )
        } else {
            format!(
                "percentile(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_percentage(percentage),
                indent_str
            )
        },
        Metric::TimeShift(metric, duration) => if depth <= 2 {
            format!(
                "timeShift({}, {})",
                pretty_print_inner(*metric, depth - 1, 0),
                pretty_print_duration(duration),
            )
        } else {
            format!(
                "timeShift(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_duration(duration),
                indent_str
            )
        },
        Metric::MovingAverage(metric, duration) => if depth <= 2 {
            format!(
                "movingAverage({}, {})",
                pretty_print_inner(*metric, depth - 1, 0),
                pretty_print_duration(duration),
            )
        } else {
            format!(
                "movingAverage(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_duration(duration),
                indent_str
            )
        },
        Metric::LinearRegression(metric, duration) => if depth <= 2 {
            format!(
                "linearRegression({}, {})",
                pretty_print_inner(*metric, depth - 1, 0),
                pretty_print_duration(duration),
            )
        } else {
            format!(
                "linearRegression(\n{},\n  {}{}\n{})",
                pretty_print_inner(*metric, depth - 1, indent + 1),
                indent_str,
                pretty_print_duration(duration),
                indent_str
            )
        },
        Metric::TimeLeftForecast(metric, duration, threshold) => format!(
            "timeLeftForecast(\n{},\n  {}{},\n  {}{}\n{})",
            pretty_print_inner(*metric, depth - 1, indent + 1),
            indent_str,
            pretty_print_duration(duration),
            indent_str,
            pretty_print_factor(threshold),
            indent_str
        ),
        Metric::Group(metrics) => format!(
            "group(\n{}\n{})",
            metrics
                .iter()
                .map(|metric| pretty_print_inner(metric.clone(), depth - 1, indent + 1))
                .collect::<Vec<_>>()
                .join(",\n"),
            indent_str
        ),
        Metric::Stack(metric) => if depth <= 2 {
            format!("stack({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("stack(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
    };
    format!("{}{}", indent_str, metric_str)
}

fn pretty_print_factor(factor: Factor) -> String {
    match factor {
        Factor::Double(s) => s,
        Factor::Fraction(nume, deno) => format!("{}/{}", nume, deno),
    }
}

fn pretty_print_percentage(percentage: Percentage) -> String {
    match percentage {
        Percentage(s) => s,
    }
}

fn pretty_print_duration(duration: Duration) -> String {
    match duration {
        Duration(s) => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cases() -> Vec<(&'static str, Metric, &'static str)> {
        vec![
            (
                "host(22CXRB3pZmu, loadavg5)",
                Metric::Host("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                "host(22CXRB3pZmu, loadavg5)",
            ),
            (
                "host ( 22CXRB3pZmu, cpu.user.percentage )",
                Metric::Host("22CXRB3pZmu".to_string(), "cpu.user.percentage".to_string()),
                "host(22CXRB3pZmu, cpu.user.percentage)",
            ),
            (
                "host('22CXRB3pZmu', memory.*)",
                Metric::Host("22CXRB3pZmu".to_string(), "memory.*".to_string()),
                "host(22CXRB3pZmu, memory.*)",
            ),
            (
                "host ( '22CXRB3pZmu', 'custom.foo.bar.*' )",
                Metric::Host("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string()),
                "host(22CXRB3pZmu, custom.foo.bar.*)",
            ),
            (
                "host ( \"22CXRB3pZmu\",\"custom.foo.bar.*\")",
                Metric::Host("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string()),
                "host(22CXRB3pZmu, custom.foo.bar.*)",
            ),
            (
                "service ( 'Blog', \"custom.access_count.*\")",
                Metric::Service("Blog".to_string(), "custom.access_count.*".to_string()),
                "service(Blog, custom.access_count.*)",
            ),
            (
                "role(Blog:db, memory.*)",
                Metric::Role("Blog".to_string(), "db".to_string(), "memory.*".to_string()),
                "role(Blog:db, memory.*)",
            ),
            (
                "role (  'Blog:  db' , 'memory.*'  ) ",
                Metric::Role("Blog".to_string(), "db".to_string(), "memory.*".to_string()),
                "role(Blog:db, memory.*)",
            ),
            (
                "roleSlots (  Blog:db , loadavg5  ) ",
                Metric::RoleSlot("Blog".to_string(), "db".to_string(), "loadavg5".to_string()),
                "roleSlots(Blog:db, loadavg5)",
            ),
            (
                "avg(group(host(22CXRB3pZmu, loadavg5), host(22CXRB3pZmv, loadavg5)))",
                Metric::Avg(Box::new(Metric::Group(vec![
                    Metric::Host("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                    Metric::Host("22CXRB3pZmv".to_string(), "loadavg5".to_string()),
                ]))),
                "avg(\n  group(\n    host(22CXRB3pZmu, loadavg5),\n    host(22CXRB3pZmv, loadavg5)\n  )\n)",
            ),
            (
                "max(role(Blog:db, loadavg5))",
                Metric::Max(Box::new(Metric::Role("Blog".to_string(), "db".to_string(), "loadavg5".to_string()))),
                "max(role(Blog:db, loadavg5))",
            ),
            (
                "min(role(Blog:db, loadavg5))",
                Metric::Min(Box::new(Metric::Role("Blog".to_string(), "db".to_string(), "loadavg5".to_string()))),
                "min(role(Blog:db, loadavg5))",
            ),
            (
                "product(group(service(Blog, foo.bar), service(Blog, foo.baz)))",
                Metric::Product(Box::new(Metric::Group(vec![
                    Metric::Service("Blog".to_string(), "foo.bar".to_string()),
                    Metric::Service("Blog".to_string(), "foo.baz".to_string()),
                ]))),
                "product(\n  group(\n    service(Blog, foo.bar),\n    service(Blog, foo.baz)\n  )\n)",
            ),
            (
                "diff(service(Blog, foo.bar), service(Blog, foo.baz))",
                Metric::Diff(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Box::new(Metric::Service("Blog".to_string(), "foo.baz".to_string())),
                ),
                "diff(\n  service(Blog, foo.bar),\n  service(Blog, foo.baz)\n)",
            ),
            (
                "divide(service(Blog, foo.bar), service(Blog, foo.baz))",
                Metric::Divide(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Box::new(Metric::Service("Blog".to_string(), "foo.baz".to_string())),
                ),
                "divide(\n  service(Blog, foo.bar),\n  service(Blog, foo.baz)\n)",
            ),
            (
                "scale ( service ( Blog , foo.bar ) , 10.0 )",
                Metric::Scale(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Factor::Double("10.0".to_string()),
                ),
                "scale(service(Blog, foo.bar), 10.0)",
            ),
            (
                "scale(scale(service('Blog', 'foo.bar'), 3.140e10), -31.4/6.25)",
                Metric::Scale(
                    Box::new(Metric::Scale(
                        Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("3.140e10".to_string()),
                    )),
                    Factor::Fraction("-31.4".to_string(), "6.25".to_string()),
                ),
                "scale(\n  scale(service(Blog, foo.bar), 3.140e10),\n  -31.4/6.25\n)",
            ),
            (
                "offset ( service ( Blog , foo.bar ) , 10.0 )",
                Metric::Offset(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Factor::Double("10.0".to_string()),
                ),
                "offset(service(Blog, foo.bar), 10.0)",
            ),
            (
                "offset(offset(service('Blog', 'foo.bar'), 3.140e10), -31.4/6.25)",
                Metric::Offset(
                    Box::new(Metric::Offset(
                        Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("3.140e10".to_string()),
                    )),
                    Factor::Fraction("-31.4".to_string(), "6.25".to_string()),
                ),
                "offset(\n  offset(service(Blog, foo.bar), 3.140e10),\n  -31.4/6.25\n)",
            ),
            (
                "percentile( role('Blog:db', 'loadavg5') , 75.5)",
                Metric::Percentile(
                    Box::new(Metric::Role("Blog".to_string(), "db".to_string(), "loadavg5".to_string())),
                    Percentage("75.5".to_string()),
                ),
                "percentile(role(Blog:db, loadavg5), 75.5)",
            ),
            (
                "timeShift(service(Blog, foo.bar), 1d)",
                Metric::TimeShift(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Duration("1d".to_string()),
                ),
                "timeShift(service(Blog, foo.bar), 1d)",
            ),
            (
                "timeShift(offset(service(Blog, foo.bar), 10.0), 1h)",
                Metric::TimeShift(
                    Box::new(Metric::Offset(
                        Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("10.0".to_string()),
                    )),
                    Duration("1h".to_string()),
                ),
                "timeShift(\n  offset(service(Blog, foo.bar), 10.0),\n  1h\n)",
            ),
            (
                "movingAverage(service(Blog, foo.bar), 1d)",
                Metric::MovingAverage(
                    Box::new(Metric::Service("Blog".to_string(), "foo.bar".to_string())),
                    Duration("1d".to_string()),
                ),
                "movingAverage(service(Blog, foo.bar), 1d)",
            ),
            (
                "linearRegression(host(22CXRB3pZmu, filesystem.drive.used), 7d)",
                Metric::LinearRegression(
                    Box::new(Metric::Host("22CXRB3pZmu".to_string(), "filesystem.drive.used".to_string())),
                    Duration("7d".to_string()),
                ),
                "linearRegression(host(22CXRB3pZmu, filesystem.drive.used), 7d)",
            ),
            (
                "scale(timeLeftForecast(host(22CXRB3pZmu, filesystem.drive.used), 3mo, 2000000000000), 1/86400)",
                Metric::Scale(
                    Box::new(Metric::TimeLeftForecast(
                        Box::new(Metric::Host("22CXRB3pZmu".to_string(), "filesystem.drive.used".to_string())),
                        Duration("3mo".to_string()),
                        Factor::Double("2000000000000".to_string()),
                    )),
                    Factor::Fraction("1".to_string(), "86400".to_string()),
                ),
                "scale(\n  timeLeftForecast(\n    host(22CXRB3pZmu, filesystem.drive.used),\n    3mo,\n    2000000000000\n  ),\n  1/86400\n)",
            ),
            (
                "group(host(22CXRB3pZmu, loadavg5), group(service(Blog, access_count.*), roleSlots(Blog:db, loadavg5)))",
                Metric::Group(vec![
                    Metric::Host("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                    Metric::Group(vec![
                        Metric::Service("Blog".to_string(), "access_count.*".to_string()),
                        Metric::RoleSlot("Blog".to_string(), "db".to_string(), "loadavg5".to_string()),
                    ]),
                ]),
                "group(\n  host(22CXRB3pZmu, loadavg5),\n  group(\n    service(Blog, access_count.*),\n    roleSlots(Blog:db, loadavg5)\n  )\n)",
            ),
            (
                "stack(role(Blog:db, loadavg5))",
                Metric::Stack(Box::new(Metric::Role("Blog".to_string(), "db".to_string(), "loadavg5".to_string()))),
                "stack(role(Blog:db, loadavg5))",
            ),
            (
                "stack(group(role(Blog:db-master, loadavg5), role(Blog:db-slave, loadavg5)))",
                Metric::Stack(Box::new(Metric::Group(vec![
                    Metric::Role("Blog".to_string(), "db-master".to_string(), "loadavg5".to_string()),
                    Metric::Role("Blog".to_string(), "db-slave".to_string(), "loadavg5".to_string()),
                ]))),
                "stack(\n  group(\n    role(Blog:db-master, loadavg5),\n    role(Blog:db-slave, loadavg5)\n  )\n)",
            ),
        ]
    }

    #[test]
    fn test_parse_metric() {
        for (source, metric, _) in test_cases() {
            let got = parse_metric(source);
            assert_eq!(got, Ok(metric));
        }
    }

    #[test]
    fn test_pretty_print() {
        for (_, metric, pretty) in test_cases() {
            let got = pretty_print(metric);
            assert_eq!(got, pretty);
            assert_eq!(pretty_print(parse_metric(got.as_ref()).unwrap()), pretty);
        }
    }
}
