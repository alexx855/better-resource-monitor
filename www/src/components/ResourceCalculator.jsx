import React, { useState, useEffect } from 'react';
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';
import { Cpu, Activity, Zap } from 'lucide-react';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs) {
  return twMerge(clsx(inputs));
}

const WORKLOADS = {
  idle: { name: 'Idle', factor: 0.1, color: '#3b82f6' },
  browsing: { name: 'Browsing', factor: 0.3, color: '#10b981' },
  dev: { name: 'Development', factor: 0.6, color: '#f59e0b' },
  gaming: { name: 'Gaming', factor: 0.8, color: '#8b5cf6' },
  rendering: { name: 'Rendering', factor: 1.0, color: '#ef4444' },
};

const ResourceCalculator = () => {
  const [load, setLoad] = useState(50);
  const [workload, setWorkload] = useState('browsing');
  const [data, setData] = useState([]);
  const [metrics, setMetrics] = useState({ power: 0, temp: 0 });

  // Persistence
  useEffect(() => {
    const saved = localStorage.getItem('calculator-state');
    if (saved) {
      try {
        const { load: sLoad, workload: sWorkload } = JSON.parse(saved);
        if (sLoad) setLoad(sLoad);
        if (sWorkload && WORKLOADS[sWorkload]) setWorkload(sWorkload);
      } catch (e) {
        console.error('Failed to load state', e);
      }
    }
  }, []);

  useEffect(() => {
    localStorage.setItem('calculator-state', JSON.stringify({ load, workload }));
  }, [load, workload]);

  // Simulation loop
  useEffect(() => {
    const interval = setInterval(() => {
      setMetrics((prev) => {
        const factor = WORKLOADS[workload].factor;
        // Simulate power based on load and workload factor + random noise
        const basePower = 5; // 5W base
        const maxPower = 40; // 40W max
        const targetPower = basePower + (maxPower - basePower) * (load / 100) * factor;
        const currentPower = targetPower + (Math.random() * 2 - 1); // Add jitter

        // Simulate temp
        const ambient = 30;
        const maxTemp = 95;
        const targetTemp = ambient + (maxTemp - ambient) * (currentPower / maxPower);

        return {
            power: Math.max(0, currentPower),
            temp: Math.max(ambient, targetTemp)
        };
      });

      setData(currentData => {
        const now = new Date();
        const timeStr = now.toLocaleTimeString();
        const newData = [...currentData, {
            time: timeStr,
            power: metrics.power,
            temp: metrics.temp
        }];
        if (newData.length > 20) newData.shift();
        return newData;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [load, workload, metrics.power, metrics.temp]); // Depend on metrics to smooth transitions if we used previous values

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-8 bg-slate-900 rounded-xl border border-slate-800 shadow-2xl text-slate-100">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Activity className="w-6 h-6 text-blue-500" />
          Resource Simulator
        </h2>
        <div className="flex gap-4 text-sm text-slate-400">
          <span className="flex items-center gap-1">
            <Zap className="w-4 h-4" /> {metrics.power.toFixed(1)} W
          </span>
          <span className="flex items-center gap-1">
            <Cpu className="w-4 h-4" /> {metrics.temp.toFixed(1)} °C
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {/* Controls */}
        <div className="space-y-6 bg-slate-800/50 p-6 rounded-lg border border-slate-700">
          <div className="space-y-3">
            <label className="text-sm font-medium text-slate-300 flex justify-between">
              CPU Load Simulation
              <span className="text-blue-400">{load}%</span>
            </label>
            <input
              type="range"
              min="0"
              max="100"
              value={load}
              onChange={(e) => setLoad(Number(e.target.value))}
              className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
            />
          </div>

          <div className="space-y-3">
            <label className="text-sm font-medium text-slate-300">
              Workload Type
            </label>
            <select
              value={workload}
              onChange={(e) => setWorkload(e.target.value)}
              className="w-full bg-slate-700 border-none rounded-lg p-2.5 text-slate-200 focus:ring-2 focus:ring-blue-500"
            >
              {Object.entries(WORKLOADS).map(([key, info]) => (
                <option key={key} value={key}>
                  {info.name}
                </option>
              ))}
            </select>
          </div>

          <div className="p-4 bg-slate-900/50 rounded border border-slate-700/50 text-sm text-slate-400">
            <p className="mb-2 font-semibold text-slate-300">Simulation Details:</p>
            <ul className="space-y-1 list-disc list-inside">
                <li>Workload Factor: {WORKLOADS[workload].factor}x</li>
                <li>Est. Battery Drain: {(metrics.power / 10).toFixed(1)}% / hour</li>
            </ul>
          </div>
        </div>

        {/* Chart */}
        <div className="h-[300px] w-full bg-slate-800/50 p-4 rounded-lg border border-slate-700 flex flex-col">
            <h3 className="text-sm font-medium text-slate-400 mb-4">Power Consumption (Real-time)</h3>
            <div className="flex-1 w-full min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={data}>
                    <defs>
                        <linearGradient id="colorPower" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                        <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                        </linearGradient>
                    </defs>
                    <XAxis dataKey="time" hide />
                    <YAxis domain={[0, 60]} hide />
                    <Tooltip
                        contentStyle={{ backgroundColor: '#1e293b', border: 'none', borderRadius: '8px' }}
                        itemStyle={{ color: '#e2e8f0' }}
                    />
                    <Area
                        type="monotone"
                        dataKey="power"
                        stroke="#3b82f6"
                        fillOpacity={1}
                        fill="url(#colorPower)"
                        isAnimationActive={false}
                    />
                    </AreaChart>
                </ResponsiveContainer>
            </div>
        </div>
      </div>
    </div>
  );
};

export default ResourceCalculator;
