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
    HostMetric(String, String),
    ServiceMetric(String, String),
    RoleMetric(String, String, String),
    RoleSlotMetric(String, String, String),
    AvgMetric(Box<Metric>),
    MaxMetric(Box<Metric>),
    MinMetric(Box<Metric>),
    ProductMetric(Box<Metric>),
    DiffMetric(Box<Metric>, Box<Metric>),
    DivideMetric(Box<Metric>, Box<Metric>),
    ScaleMetric(Box<Metric>, Factor),
    OffsetMetric(Box<Metric>, Factor),
    PercentileMetric(Box<Metric>, Percentage),
    TimeShiftMetric(Box<Metric>, Duration),
    GroupMetric(Vec<Metric>),
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

pub fn parse_metric(src: &str) -> Result<Metric, String> {
    let mut pairs = MetricParser::parse_str(Rule::whole_metrics, src).map_err(|e| format!("{}", e))?;
    convert_metrics(pairs.next().ok_or("metrics")?.into_inner().next().unwrap())
}

macro_rules! arg {
    ($pairs:expr) => {
        $pairs.next().unwrap().into_inner().next().unwrap().as_str().to_string()
    }
}

fn convert_metrics<I: Input>(pair: Pair<Rule, I>) -> Result<Metric, String> {
    match pair.as_rule() {
        Rule::host_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::HostMetric(arg!(inner), arg!(inner)))
        }
        Rule::service_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::ServiceMetric(arg!(inner), arg!(inner)))
        }
        Rule::role_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = inner.next().unwrap().into_inner().next().unwrap().into_inner();
            let service_name = role_full_name.next().unwrap().as_str().to_string();
            let role_name = role_full_name.next().unwrap().as_str().to_string();
            Ok(Metric::RoleMetric(service_name, role_name, arg!(inner)))
        }
        Rule::role_slot_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = inner.next().unwrap().into_inner().next().unwrap().into_inner();
            let service_name = role_full_name.next().unwrap().as_str().to_string();
            let role_name = role_full_name.next().unwrap().as_str().to_string();
            Ok(Metric::RoleSlotMetric(service_name, role_name, arg!(inner)))
        }
        Rule::avg_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::AvgMetric(Box::new(convert_metrics(inner.next().unwrap())?)))
        }
        Rule::max_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::MaxMetric(Box::new(convert_metrics(inner.next().unwrap())?)))
        }
        Rule::min_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::MinMetric(Box::new(convert_metrics(inner.next().unwrap())?)))
        }
        Rule::product_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::ProductMetric(Box::new(convert_metrics(inner.next().unwrap())?)))
        }
        Rule::diff_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::DiffMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                Box::new(convert_metrics(inner.next().unwrap())?),
            ))
        }
        Rule::divide_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::DivideMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                Box::new(convert_metrics(inner.next().unwrap())?),
            ))
        }
        Rule::scale_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::ScaleMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                convert_factor(inner.next().unwrap().into_inner().next().unwrap())?,
            ))
        }
        Rule::offset_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::OffsetMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                convert_factor(inner.next().unwrap().into_inner().next().unwrap())?,
            ))
        }
        Rule::percentile_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::PercentileMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                convert_percentage(inner.next().unwrap())?,
            ))
        }
        Rule::time_shift_metric => {
            let mut inner = pair.into_inner();
            Ok(Metric::TimeShiftMetric(
                Box::new(convert_metrics(inner.next().unwrap())?),
                convert_duration(inner.next().unwrap().into_inner().next().unwrap())?,
            ))
        }
        Rule::group_metric => {
            let mut metrics = Vec::new();
            for r in pair.into_inner() {
                metrics.push(convert_metrics(r)?);
            }
            Ok(Metric::GroupMetric(metrics))
        }
        Rule::metrics => convert_metrics(pair.into_inner().next().unwrap()),
        _ => unreachable!(),
    }
}

fn convert_factor<I: Input>(pair: Pair<Rule, I>) -> Result<Factor, String> {
    match pair.as_rule() {
        Rule::double => Ok(Factor::Double(pair.as_str().to_string())),
        Rule::fraction => {
            let mut inner = pair.into_inner();
            Ok(Factor::Fraction(
                inner.next().unwrap().as_str().to_string(),
                inner.next().unwrap().as_str().to_string(),
            ))
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
        Metric::AvgMetric(metric) => 1 + calc_depth(*metric),
        Metric::MaxMetric(metric) => 1 + calc_depth(*metric),
        Metric::MinMetric(metric) => 1 + calc_depth(*metric),
        Metric::ProductMetric(metric) => 1 + calc_depth(*metric),
        Metric::DiffMetric(metric1, metric2) => 1 + vec![calc_depth(*metric1), calc_depth(*metric2)].iter().max().unwrap(),
        Metric::DivideMetric(metric1, metric2) => 1 + vec![calc_depth(*metric1), calc_depth(*metric2)].iter().max().unwrap(),
        Metric::ScaleMetric(metric, _) => 1 + calc_depth(*metric),
        Metric::OffsetMetric(metric, _) => 1 + calc_depth(*metric),
        Metric::PercentileMetric(metric, _) => 1 + calc_depth(*metric),
        Metric::TimeShiftMetric(metric, _) => 1 + calc_depth(*metric),
        Metric::GroupMetric(metrics) => 1 + metrics.iter().map(|metric| calc_depth(metric.clone())).max().unwrap(),
        _ => 1,
    }
}

fn pretty_print_inner(metric: Metric, depth: u64, indent: usize) -> String {
    let indent_str = " ".repeat(indent * 2);
    let metric_str = match metric {
        Metric::HostMetric(host_id, metric_name) => format!("host({}, {})", host_id, metric_name),
        Metric::ServiceMetric(service_name, metric_name) => format!("service({}, {})", service_name, metric_name),
        Metric::RoleMetric(service_name, role_name, metric_name) => format!("role({}:{}, {})", service_name, role_name, metric_name),
        Metric::RoleSlotMetric(service_name, role_name, metric_name) => format!("roleSlots({}:{}, {})", service_name, role_name, metric_name),
        Metric::AvgMetric(metric) => if depth <= 2 {
            format!("avg({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("avg(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::MaxMetric(metric) => if depth <= 2 {
            format!("max({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("max(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::MinMetric(metric) => if depth <= 2 {
            format!("min({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("min(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::ProductMetric(metric) => if depth <= 2 {
            format!("product({})", pretty_print_inner(*metric, depth - 1, 0))
        } else {
            format!("product(\n{}\n{})", pretty_print_inner(*metric, depth - 1, indent + 1), indent_str)
        },
        Metric::DiffMetric(metric1, metric2) => format!(
            "diff(\n{},\n{}\n{})",
            pretty_print_inner(*metric1, depth - 1, indent + 1),
            pretty_print_inner(*metric2, depth - 1, indent + 1),
            indent_str
        ),
        Metric::DivideMetric(metric1, metric2) => format!(
            "divide(\n{},\n{}\n{})",
            pretty_print_inner(*metric1, depth - 1, indent + 1),
            pretty_print_inner(*metric2, depth - 1, indent + 1),
            indent_str
        ),
        Metric::ScaleMetric(metric, factor) => if depth <= 2 {
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
        Metric::OffsetMetric(metric, factor) => if depth <= 2 {
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
        Metric::PercentileMetric(metric, percentage) => if depth <= 2 {
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
        Metric::TimeShiftMetric(metric, duration) => if depth <= 2 {
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
        Metric::GroupMetric(metrics) => format!(
            "group(\n{}\n{})",
            metrics
                .iter()
                .map(|metric| pretty_print_inner(metric.clone(), depth - 1, indent + 1))
                .collect::<Vec<_>>()
                .join(",\n"),
            indent_str
        ),
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
                Metric::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                "host(22CXRB3pZmu, loadavg5)",
            ),
            (
                "host ( 22CXRB3pZmu, cpu.user.percentage )",
                Metric::HostMetric("22CXRB3pZmu".to_string(), "cpu.user.percentage".to_string()),
                "host(22CXRB3pZmu, cpu.user.percentage)",
            ),
            (
                "host('22CXRB3pZmu', memory.*)",
                Metric::HostMetric("22CXRB3pZmu".to_string(), "memory.*".to_string()),
                "host(22CXRB3pZmu, memory.*)",
            ),
            (
                "host ( '22CXRB3pZmu', 'custom.foo.bar.*' )",
                Metric::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string()),
                "host(22CXRB3pZmu, custom.foo.bar.*)",
            ),
            (
                "host ( \"22CXRB3pZmu\",\"custom.foo.bar.*\")",
                Metric::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string()),
                "host(22CXRB3pZmu, custom.foo.bar.*)",
            ),
            (
                "service ( 'Blog', \"custom.access_count.*\")",
                Metric::ServiceMetric("Blog".to_string(), "custom.access_count.*".to_string()),
                "service(Blog, custom.access_count.*)",
            ),
            (
                "role(Blog:db, memory.*)",
                Metric::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string()),
                "role(Blog:db, memory.*)",
            ),
            (
                "role (  'Blog:  db' , 'memory.*'  ) ",
                Metric::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string()),
                "role(Blog:db, memory.*)",
            ),
            (
                "roleSlots (  Blog:db , loadavg5  ) ",
                Metric::RoleSlotMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string()),
                "roleSlots(Blog:db, loadavg5)",
            ),
            (
                "avg(group(host(22CXRB3pZmu, loadavg5), host(22CXRB3pZmv, loadavg5)))",
                Metric::AvgMetric(Box::new(Metric::GroupMetric(vec![
                    Metric::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                    Metric::HostMetric("22CXRB3pZmv".to_string(), "loadavg5".to_string()),
                ]))),
                "avg(\n  group(\n    host(22CXRB3pZmu, loadavg5),\n    host(22CXRB3pZmv, loadavg5)\n  )\n)",
            ),
            (
                "max(role(Blog:db, loadavg5))",
                Metric::MaxMetric(Box::new(Metric::RoleMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string()))),
                "max(role(Blog:db, loadavg5))",
            ),
            (
                "min(role(Blog:db, loadavg5))",
                Metric::MinMetric(Box::new(Metric::RoleMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string()))),
                "min(role(Blog:db, loadavg5))",
            ),
            (
                "product(group(service(Blog, foo.bar), service(Blog, foo.baz)))",
                Metric::ProductMetric(Box::new(Metric::GroupMetric(vec![
                    Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string()),
                    Metric::ServiceMetric("Blog".to_string(), "foo.baz".to_string()),
                ]))),
                "product(\n  group(\n    service(Blog, foo.bar),\n    service(Blog, foo.baz)\n  )\n)",
            ),
            (
                "diff(service(Blog, foo.bar), service(Blog, foo.baz))",
                Metric::DiffMetric(
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.baz".to_string())),
                ),
                "diff(\n  service(Blog, foo.bar),\n  service(Blog, foo.baz)\n)",
            ),
            (
                "divide(service(Blog, foo.bar), service(Blog, foo.baz))",
                Metric::DivideMetric(
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.baz".to_string())),
                ),
                "divide(\n  service(Blog, foo.bar),\n  service(Blog, foo.baz)\n)",
            ),
            (
                "scale ( service ( Blog , foo.bar ) , 10.0 )",
                Metric::ScaleMetric(
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                    Factor::Double("10.0".to_string()),
                ),
                "scale(service(Blog, foo.bar), 10.0)",
            ),
            (
                "scale(scale(service('Blog', 'foo.bar'), 3.140e10), -31.4/6.25)",
                Metric::ScaleMetric(
                    Box::new(Metric::ScaleMetric(
                        Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("3.140e10".to_string()),
                    )),
                    Factor::Fraction("-31.4".to_string(), "6.25".to_string()),
                ),
                "scale(\n  scale(service(Blog, foo.bar), 3.140e10),\n  -31.4/6.25\n)",
            ),
            (
                "offset ( service ( Blog , foo.bar ) , 10.0 )",
                Metric::OffsetMetric(
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                    Factor::Double("10.0".to_string()),
                ),
                "offset(service(Blog, foo.bar), 10.0)",
            ),
            (
                "offset(offset(service('Blog', 'foo.bar'), 3.140e10), -31.4/6.25)",
                Metric::OffsetMetric(
                    Box::new(Metric::OffsetMetric(
                        Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("3.140e10".to_string()),
                    )),
                    Factor::Fraction("-31.4".to_string(), "6.25".to_string()),
                ),
                "offset(\n  offset(service(Blog, foo.bar), 3.140e10),\n  -31.4/6.25\n)",
            ),
            (
                "percentile( role('Blog:db', 'loadavg5') , 75.5)",
                Metric::PercentileMetric(
                    Box::new(Metric::RoleMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string())),
                    Percentage("75.5".to_string()),
                ),
                "percentile(role(Blog:db, loadavg5), 75.5)",
            ),
            (
                "timeShift(service(Blog, foo.bar), 1d)",
                Metric::TimeShiftMetric(
                    Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                    Duration("1d".to_string()),
                ),
                "timeShift(service(Blog, foo.bar), 1d)",
            ),
            (
                "timeShift(offset(service(Blog, foo.bar), 10.0), 1h)",
                Metric::TimeShiftMetric(
                    Box::new(Metric::OffsetMetric(
                        Box::new(Metric::ServiceMetric("Blog".to_string(), "foo.bar".to_string())),
                        Factor::Double("10.0".to_string()),
                    )),
                    Duration("1h".to_string()),
                ),
                "timeShift(\n  offset(service(Blog, foo.bar), 10.0),\n  1h\n)",
            ),
            (
                "group(host(22CXRB3pZmu, loadavg5), group(service(Blog, access_count.*), roleSlots(Blog:db, loadavg5)))",
                Metric::GroupMetric(vec![
                    Metric::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                    Metric::GroupMetric(vec![
                        Metric::ServiceMetric("Blog".to_string(), "access_count.*".to_string()),
                        Metric::RoleSlotMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string()),
                    ]),
                ]),
                "group(\n  host(22CXRB3pZmu, loadavg5),\n  group(\n    service(Blog, access_count.*),\n    roleSlots(Blog:db, loadavg5)\n  )\n)",
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
