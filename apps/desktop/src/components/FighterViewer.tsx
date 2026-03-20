import React, { useEffect, useState, useRef } from 'react';
import { useStore } from '../store/useStore';

export const FighterViewer: React.FC = () => {
  const { 
    fighters, 
    loadFighterList, 
    selectedFighterId, 
    selectFighter, 
    poses,
    renderPose 
  } = useStore();

  const [selectedPoseIndex, setSelectedPoseIndex] = useState<number>(0);
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    loadFighterList();
  }, []);

  useEffect(() => {
    if (selectedFighterId !== null) {
      updatePose(0);
    }
  }, [selectedFighterId]);

  const updatePose = async (poseIndex: number) => {
    if (selectedFighterId === null) return;
    setLoading(true);
    try {
      const bytes = await renderPose(selectedFighterId, poseIndex);
      const blob = new Blob([bytes], { type: 'image/png' });
      const url = URL.createObjectURL(blob);
      setImageSrc(url);
      setSelectedPoseIndex(poseIndex);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-900 text-white p-6">
      <h1 className="text-2xl font-bold mb-6 text-blue-400">Fighter Graphics Viewer</h1>
      
      <div className="flex gap-6 h-full overflow-hidden">
        {/* Sidebar: Fighter List */}
        <div className="w-64 flex flex-col gap-2 overflow-y-auto pr-2 border-r border-slate-700">
          <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-2">Fighters</h2>
          {fighters.map((f) => (
            <button
              key={f.id}
              onClick={() => selectFighter(f.id)}
              className={`p-3 text-left rounded transition ${
                selectedFighterId === f.id 
                ? 'bg-blue-600 shadow-lg shadow-blue-900/50' 
                : 'hover:bg-slate-800'
              }`}
            >
              {f.name}
            </button>
          ))}
        </div>

        {/* Main Area */}
        <div className="flex-1 flex flex-col gap-6">
          {selectedFighterId === null ? (
            <div className="flex-1 flex items-center justify-center text-slate-500 italic">
              Select a fighter to view poses
            </div>
          ) : (
            <>
              {/* Pose Selection & Tools */}
              <div className="flex flex-col gap-4">
                <div className="flex items-center justify-between bg-slate-800 p-4 rounded-lg">
                  <div className="flex items-center gap-4">
                    <span className="text-slate-400 font-medium">Pose:</span>
                    <select
                      value={selectedPoseIndex}
                      onChange={(e) => updatePose(parseInt(e.target.value))}
                      className="bg-slate-700 border border-slate-600 rounded px-3 py-1 outline-none focus:border-blue-500"
                    >
                      {poses.map((p, idx) => (
                        <option key={idx} value={idx}>
                          Pose {idx} (Addr: 0x{p.data_addr.toString(16).toUpperCase()})
                        </option>
                      ))}
                    </select>
                  </div>
                  
                  <div className="flex gap-2">
                    <button 
                       disabled={selectedPoseIndex <= 0}
                       onClick={() => updatePose(selectedPoseIndex - 1)}
                       className="px-4 py-1 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
                    >
                      Prev
                    </button>
                    <button 
                       disabled={selectedPoseIndex >= poses.length - 1}
                       onClick={() => updatePose(selectedPoseIndex + 1)}
                       className="px-4 py-1 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
                    >
                      Next
                    </button>
                  </div>
                </div>
              </div>

              {/* Rendering Canvas */}
              <div className="flex-1 relative bg-slate-950 rounded-xl border border-slate-800 flex items-center justify-center overflow-hidden">
                {loading && (
                    <div className="absolute inset-0 z-10 bg-slate-950/50 flex items-center justify-center">
                        <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
                    </div>
                )}
                
                {imageSrc ? (
                  <div className="relative group">
                    <img 
                      src={imageSrc} 
                      alt="Fighter Pose" 
                      className="pixelated scale-[2] transform-gpu transition shadow-2xl" 
                      style={{ 
                        imageRendering: 'pixelated',
                        maxWidth: '512px',
                        maxHeight: '512px'
                       }}
                    />
                    <div className="absolute -bottom-10 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition text-xs text-slate-500">
                      256x256 Canvas (2x Zoom)
                    </div>
                  </div>
                ) : (
                   <div className="text-slate-700">No pose data</div>
                )}
              </div>
            </>
          )}
        </div>
      </div>
      
      <style>{`
        .pixelated {
          image-rendering: -moz-crisp-edges;
          image-rendering: -webkit-crisp-edges;
          image-rendering: pixelated;
          image-rendering: crisp-edges;
        }
      `}</style>
    </div>
  );
};
