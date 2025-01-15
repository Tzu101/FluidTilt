<script setup lang="ts">
import { onMounted, ref, onBeforeUnmount } from "vue";
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core';
import { debounce } from 'lodash';

const canvasRef = ref<HTMLCanvasElement | null>(null);
function updateCanvas() {
  if (!canvasRef.value){
    return;
  }

  canvasRef.value.width = gridCols * cellSize;
  canvasRef.value.height = gridRows * cellSize;
}

const cellSize = 10;  // Cell size in pixels
let gridCols= 0;
let gridRows = 0;
let gridPadding = 50;
const onResize = debounce(() => {
  gridCols = Math.floor((window.innerWidth - gridPadding) / cellSize);
  gridRows = Math.floor((window.innerHeight - gridPadding) / cellSize);
  console.log(gridCols, gridRows);
  updateCanvas();
}, 250);

const BLUE_BASE = 45;
function drawGrid(grid: number[][]) {
  const ctx = canvasRef.value!.getContext('2d');
  if (!ctx) {
    return;
  }

  for (let row = 0; row < gridRows; row++) {
    for (let col = 0; col < gridCols; col++) {
      const blueGradient = BLUE_BASE * (grid[row][col] + 1);
      const greenGradient = blueGradient * 2 / 3;
      ctx.fillStyle = `rgb(${0}, ${greenGradient}, ${blueGradient})`;
      ctx.fillRect(col * cellSize, row * cellSize, cellSize, cellSize);
    }
  }
}

onMounted(() => {
  onResize();
  updateCanvas();
  window.addEventListener('resize', onResize);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize);
});


type FluidGrid = {
  data: number[][];
};
const UPDATE_GRID_EVENT = "update_grid";

listen<FluidGrid>(UPDATE_GRID_EVENT, (event) => {
  drawGrid(event.payload.data);
});

async function startSimulation() {
  await invoke('start_fluid_simulation', { rows: gridRows, cols: gridCols });
}

async function stopSimulation() {
  await invoke('stop_fluid_simulation');
}
</script>

<template>
  <canvas ref="canvasRef"></canvas>
  <button @click="startSimulation">Start simulation</button>
  <button @click="stopSimulation">Stop simulation</button>
</template>

<style scoped lang="scss">
canvas {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);

  // Scss supports comments
}
</style>

<style lang="scss">
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;

  font-family: monospace;
}

:root {
  overflow: hidden;

  background-color: rgb(0, 30, 45);
}
</style>