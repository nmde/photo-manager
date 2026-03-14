<!-- eslint-disable @typescript-eslint/no-invalid-this -->
<script setup lang="ts">
  import type { Tag } from '@/classes/Tag';
  import {
    type BaseType,
    drag,
    forceLink,
    type ForceLink,
    forceManyBody,
    forceSimulation,
    forceX,
    forceY,
    select,
    type Selection,
    type Simulation,
    type SimulationNodeDatum,
    zoom,
  } from 'd3';

  type Node = SimulationNodeDatum & { label: string; color: string };
  type Link = { source: Node; target: Node };

  const props = defineProps<{
    data: Record<string, Tag>;
    width: number;
    height: number;
    changed: number;
  }>();

  const containerEl = ref<SVGElement | null>();

  const graph = computed(() => {
    const nodes: Record<string, Node> = {};
    const links: Link[] = [];

    const hasDependents = new Set<string>();
    for (const [name, tag] of Object.entries(props.data)) {
      if (tag.prereqs.length > 0) {
        nodes[name] = {
          label: name,
          color: tag.color,
        };
        for (const p of tag.prereqs) {
          hasDependents.add(p);
        }
      }
    }

    for (const p of hasDependents) {
      if (!nodes[p]) {
        nodes[p] = {
          label: p,
          color: props.data[p]?.color ?? '',
        };
      }
    }

    for (const tag of Object.values(props.data)) {
      for (const source of tag.prereqs) {
        const target = tag.name;
        if (nodes[source] && nodes[target]) {
          links.push({
            source: nodes[source],
            target: nodes[target],
          });
        }
      }
    }

    // The computed property doesn't get re-evaluated if this log isn't here :(
    console.log(Object.values(nodes));

    return { nodes: Object.values(nodes), links };
  });

  function tick(
    nodes: Selection<SVGCircleElement | BaseType, Node, SVGGElement, unknown>,
    links: Selection<SVGLineElement | BaseType, Link, SVGGElement, unknown>,
    labels: Selection<SVGTextElement | BaseType, Node, SVGGElement, unknown>,
  ) {
    nodes.attr('cx', d => d.x ?? 0).attr('cy', d => d.y ?? 0);
    links
      .attr('x1', d => d.source.x ?? 0)
      .attr('y1', d => d.source.y ?? 0)
      .attr('x2', d => d.target.x ?? 0)
      .attr('y2', d => d.target.y ?? 0);
    labels
      .attr('x', function (d) {
        return (d.x ?? 0) - (this as SVGTextElement).getBBox().width / 2;
      })
      .attr('y', function (d) {
        return (d.y ?? 0) + (this as SVGTextElement).getBBox().height / 4;
      });
  }

  let simulation: Simulation<Node, undefined> | null = null;
  let _transform = '';
  function render() {
    if (simulation !== null) {
      simulation.stop();
    }
    if (containerEl.value) {
      const container = select(containerEl.value)
        .attr('viewBox', [-props.width / 2, -props.height / 2, props.width, props.height])
        .attr('width', props.width)
        .attr('height', props.height);

      container.selectChildren().remove();

      container.call(
        zoom<SVGElement, unknown>()
          .extent([
            [0, 0],
            [props.width, props.height],
          ])
          .on('zoom', ({ transform }) => {
            _transform = transform;
            container.selectAll('g').attr('transform', transform);
          }),
      );

      const links = container
        .append('g')
        .selectAll('line')
        .data(graph.value.links)
        .join('line')
        .attr(
          'class',
          d => `tag-graph-link${d.target.color.length === 0 ? ' tag-graph-link--default' : ''}`,
        )
        .attr('stroke', d => d.target.color);

      const nodes = container
        .append('g')
        .selectAll('circle')
        .data(graph.value.nodes)
        .join(enter =>
          enter
            .append('circle')
            .attr('r', 5)
            .attr(
              'class',
              d => `tag-graph-node${d.color.length === 0 ? ' tag-graph-node--default' : ''}`,
            )
            .attr('fill', d => d.color),
        );

      const labels = container
        .append('g')
        .selectAll('text')
        .data(graph.value.nodes)
        .join(enter =>
          enter
            .append('text')
            .text(d => d.label)
            .attr(
              'class',
              d => `tag-graph-label${d.color.length === 0 ? ' tag-graph-label--default' : ''}`,
            )
            .attr('fill', d => d.color)
            .call(
              drag<SVGTextElement, Node>()
                .on('start', (event, d) => {
                  console.log(event, d);
                  if (!event.active) {
                    simulation?.alphaTarget(0.3).restart();
                  }
                  d.fx = d.x;
                  d.fy = d.y;
                })
                .on('drag', (event, d) => {
                  d.fx = event.x;
                  d.fy = event.y;
                })
                .on('end', (event, d) => {
                  if (!event.active) {
                    simulation?.alphaTarget(0);
                  }
                  d.fx = null;
                  d.fy = null;
                }),
            ),
        );

      container.selectAll('g').attr('transform', _transform);

      simulation = forceSimulation<Node>()
        .force('charge', forceManyBody().strength(-100))
        .force('link', forceLink())
        .force('x', forceX())
        .force('y', forceY())
        .on('tick', () => tick(nodes, links, labels));

      simulation.nodes(graph.value.nodes);
      simulation.force<ForceLink<Node, Link>>('link')?.links(graph.value.links);
      simulation.alpha(1).restart().tick();
      tick(nodes, links, labels);
    }
  }

  onMounted(render);

  watch(() => props.changed, render);
</script>

<template>
  <svg ref="containerEl" />
</template>
