extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::Pair;
use pest::inputs::Input;

#[derive(Parser)]
#[grammar = "graph.pest"]
pub struct GraphParser;

#[derive(Debug, PartialEq)]
pub enum Graph {
    HostMetric(String, String),
    ServiceMetric(String, String),
    RoleMetric(String, String, String),
    RoleSlotMetric(String, String, String),
    GroupMetric(Vec<Graph>),
}

pub fn parse_graph(src: &str) -> Result<Graph, String> {
    let mut pairs = GraphParser::parse_str(Rule::graph, src).map_err(|e| format!("{}", e))?;
    convert_metrics(pairs.next().ok_or("graph")?.into_inner().next().unwrap())
}

fn convert_metrics<I: Input>(pair: Pair<Rule, I>) -> Result<Graph, String> {
    match pair.as_rule() {
        Rule::host_metric => {
            let mut inner = pair.into_inner();
            let host_id = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            let metric_name = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            Ok(Graph::HostMetric(host_id, metric_name))
        }
        Rule::service_metric => {
            let mut inner = pair.into_inner();
            let service_name = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            let metric_name = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            Ok(Graph::ServiceMetric(service_name, metric_name))
        }
        Rule::role_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = inner.next().unwrap().into_inner().next().unwrap().into_inner();
            let service_name = role_full_name.next().unwrap().as_str().to_string();
            let role_name = role_full_name.next().unwrap().as_str().to_string();
            let metric_name = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            Ok(Graph::RoleMetric(service_name, role_name, metric_name))
        }
        Rule::role_slot_metric => {
            let mut inner = pair.into_inner();
            let mut role_full_name = inner.next().unwrap().into_inner().next().unwrap().into_inner();
            let service_name = role_full_name.next().unwrap().as_str().to_string();
            let role_name = role_full_name.next().unwrap().as_str().to_string();
            let metric_name = inner.next().unwrap().into_inner().next().unwrap().as_str().to_string();
            Ok(Graph::RoleSlotMetric(service_name, role_name, metric_name))
        }
        Rule::group_metric => {
            let mut metrics = Vec::new();
            for r in pair.into_inner() {
                metrics.push(convert_metrics(r)?);
            }
            Ok(Graph::GroupMetric(metrics))
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
            vec![("host(22CXRB3pZmu, loadavg5)", Graph::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string())),
                 ("host(22CXRB3pZmu, cpu.user.percentage)",
                  Graph::HostMetric("22CXRB3pZmu".to_string(), "cpu.user.percentage".to_string())),
                 ("host('22CXRB3pZmu', memory.*)", Graph::HostMetric("22CXRB3pZmu".to_string(), "memory.*".to_string())),
                 ("host ( '22CXRB3pZmu', 'custom.foo.bar.*' )",
                  Graph::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string())),
                 ("host ( \"22CXRB3pZmu\",\"custom.foo.bar.*\")",
                  Graph::HostMetric("22CXRB3pZmu".to_string(), "custom.foo.bar.*".to_string())),
                 ("service ( 'Blog', \"custom.access_count.*\")",
                  Graph::ServiceMetric("Blog".to_string(), "custom.access_count.*".to_string())),
                 ("role(Blog:db, memory.*)",
                  Graph::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string())),
                 ("role (  'Blog:  db' , 'memory.*'  ) ",
                  Graph::RoleMetric("Blog".to_string(), "db".to_string(), "memory.*".to_string())),
                 ("roleSlots (  Blog:db , loadavg5  ) ",
                  Graph::RoleSlotMetric("Blog".to_string(), "db".to_string(), "loadavg5".to_string())),
                 ("group(host(22CXRB3pZmu, loadavg5), group(service(Blog, access_count.*), roleSlots(Blog:db, loadavg5)))",
                  Graph::GroupMetric(vec![Graph::HostMetric("22CXRB3pZmu".to_string(), "loadavg5".to_string()),
                                          Graph::GroupMetric(vec![Graph::ServiceMetric("Blog".to_string(),
                                                                                       "access_count.*".to_string()),
                                                                  Graph::RoleSlotMetric("Blog".to_string(),
                                                                                        "db".to_string(),
                                                                                        "loadavg5".to_string())])]))];
        for (source, expected) in sources {
            let got = parse_graph(source);
            assert_eq!(got, Ok(expected));
        }
    }
}
