extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::Pair;
use pest::inputs::Input;

#[derive(Parser)]
#[grammar = "metrics.pest"]
pub struct GraphParser;

#[derive(Debug, PartialEq)]
pub enum Metric {
    HostMetric(String, String),
    ServiceMetric(String, String),
    RoleMetric(String, String, String),
    RoleSlotMetric(String, String, String),
    AvgMetric(Box<Metric>),
    GroupMetric(Vec<Metric>),
}

pub fn parse_graph(src: &str) -> Result<Metric, String> {
    let mut pairs = GraphParser::parse_str(Rule::whole_metrics, src).map_err(|e| format!("{}", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_graph() {
        let sources =
            vec![("host(22CXRB3pZmu, loadavg5)", Metric::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string())),
                 ("host(22CXRB3pZmu, cpu.user.percentage)",
                  Metric::HostMetric("22CXRB3pZmu".to_string(), "cpu.user.percentage".to_string())),
                 ("host('22CXRB3pZmu', memory.*)", Metric::HostMetric("22CXRB3pZmu".to_string(), "memory.*".to_string())),
                 ("host ( '22CXRB3pZmu', 'custom.foo.bar.*' )",
                  Metric::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string())),
                 ("host ( \"22CXRB3pZmu\",\"custom.foo.bar.*\")",
                  Metric::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string())),
                 ("service ( 'Blog', \"custom.access_count.*\")",
                  Metric::ServiceMetric("Blog".to_string(), "custom.access_count.*".to_string())),
                 ("role(Blog:db, memory.*)",
                  Metric::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string())),
                 ("role (  'Blog:  db' , 'memory.*'  ) ",
                  Metric::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string())),
                 ("roleSlots (  Blog:db , loadavg5  ) ",
                  Metric::RoleSlotMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string())),
                 ("avg(group(host(22CXRB3pZmu, loadavg5), host(22CXRB3pZmv, loadavg5)))",
                  Metric::AvgMetric(Box::new(Metric::GroupMetric(vec![Metric::HostMetric("22CXRB3pZmu".to_string(),
                                                                                         "loadavg5".to_string()),
                                                                      Metric::HostMetric("22CXRB3pZmv".to_string(),
                                                                                         "loadavg5".to_string())])))),
                 ("group(host(22CXRB3pZmu, loadavg5), group(service(Blog, access_count.*), roleSlots(Blog:db, loadavg5)))",
                  Metric::GroupMetric(vec![Metric::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                                           Metric::GroupMetric(vec![Metric::ServiceMetric("Blog".to_string(),
                                                                                          "access_count.*"
                                                                                              .to_string()),
                                                                    Metric::RoleSlotMetric("Blog".to_string(),
                                                                                           "db".to_string(),
                                                                                           "loadavg5".to_string())])]))];
        for (source, expected) in sources {
            let got = parse_graph(source);
            assert_eq!(got, Ok(expected));
        }
    }
}
