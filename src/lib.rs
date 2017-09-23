extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "graph.pest"]
pub struct GraphParser;

#[derive(Debug, PartialEq)]
pub enum Graph {
    MetricName(String),
}

pub fn parse_graph(src: &str) -> Result<Graph, String> {
    let mut pairs = GraphParser::parse_str(Rule::graph, src).map_err(|e| format!("{}", e))?;
    if let Some(pair) = pairs.next().unwrap().into_inner().next() {
        match pair.as_rule() {
            Rule::metric_name => Ok(Graph::MetricName(pair.into_inner().next().unwrap().as_str().to_string())),
            _ => unreachable!(),
        }
    } else {
        Err("err".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metric_name() {
        let sources = vec![("loadavg5", Graph::MetricName("loadavg5".to_string())),
                           ("cpu.user.percentage", Graph::MetricName("cpu.user.percentage".to_string())),
                           ("memory.*", Graph::MetricName("memory.*".to_string())),
                           ("'custom.foo.bar.*'", Graph::MetricName("custom.foo.bar.*".to_string())),
                           ("\"custom.foo.bar.*\"", Graph::MetricName("custom.foo.bar.*".to_string()))];
        for (source, expected) in sources {
            let got = parse_graph(source);
            assert!(got.is_ok());
            assert_eq!(got.unwrap(), expected);
        }
    }
}
