import React, { useState, useEffect } from 'react';
import { request as invoke } from '../utils/request';
import { 
    Download, CheckCircle2, XCircle, Loader2, Save, Settings as SettingsIcon, 
    Play, Square, Trash2, RefreshCw, Terminal, Cpu, ShieldCheck, Server, Sparkles, Folder
} from 'lucide-react';
import { toast } from 'sonner';

interface BootstrapperProgram {
    id: string;
    name: string;
    description: string;
    download_url: string | null;
    local_path: string | null;
}

interface BootstrapperConfig {
    programs: BootstrapperProgram[];
}

export default function Bootstrapper() {
    const [config, setConfig] = useState<BootstrapperConfig | null>(null);
    const [installing, setInstalling] = useState<Record<string, boolean>>({});
    const [selectedNode, setSelectedNode] = useState<string>('localhost');
    const [status, setStatus] = useState<Record<string, { isInstalled: boolean, detectedPath: string | null }>>({});
    const [editMode, setEditMode] = useState<string | null>(null);
    const [editForm, setEditForm] = useState<Partial<BootstrapperProgram>>({});
    const [actionLoading, setActionLoading] = useState<Record<string, boolean>>({});
    const [logs, setLogs] = useState<string[]>([]);
    const [selectedCategory, setSelectedCategory] = useState<string>('all');

    const addLog = (msg: string) => {
        const time = new Date().toLocaleTimeString();
        setLogs(prev => [`[${time}] ${msg}`, ...prev.slice(0, 49)]);
    };

    const loadConfig = async () => {
        try {
            addLog("Cargando matriz de programas...");
            const data = await invoke<BootstrapperConfig>('bootstrapper_get_config');
            setConfig(data);
            addLog(`Matriz cargada con ${data.programs.length} programas registrados.`);
            data.programs.forEach(prog => handleVerify(prog.id, selectedNode));
        } catch (error) {
            console.error(error);
            addLog(`Error al cargar la configuración: ${error}`);
            toast.error("No se pudo cargar la configuración del Bootstrapper.");
            setConfig({ programs: [] });
        }
    };

    useEffect(() => {
        loadConfig();
    }, []);

    useEffect(() => {
        if (config) {
            setStatus({});
            addLog(`Conmutando nodo de destino a: ${selectedNode}`);
            config.programs.forEach(prog => handleVerify(prog.id, selectedNode));
        }
    }, [selectedNode]);

    const handleVerify = async (id: string, node: string = selectedNode) => {
        try {
            const result = await invoke<{ installed: boolean, detected_path: string | null }>('bootstrapper_verify_program', { programId: id, targetNode: node });
            setStatus(prev => ({ ...prev, [id]: { isInstalled: result.installed, detectedPath: result.detected_path } }));
            if (result.installed) {
                addLog(`✓ ${id}: Detectado en ${result.detected_path}`);
            } else {
                addLog(`✗ ${id}: No encontrado en ${node}`);
            }
        } catch (error) {
            console.error(error);
            setStatus(prev => ({ ...prev, [id]: { isInstalled: false, detectedPath: null } }));
            addLog(`Fallo al verificar ${id}: ${error}`);
        }
    };

    const handleInstall = async (id: string) => {
        setInstalling(prev => ({ ...prev, [id]: true }));
        addLog(`Iniciando descarga e instalación autónoma de '${id}' en ${selectedNode}...`);
        toast.info(`Iniciando instalación de ${id} en ${selectedNode}...`);
        try {
            if (selectedNode === 'localhost') {
                await invoke('bootstrapper_install_program', { programId: id });
            } else {
                await invoke('bootstrapper_deploy_ag_cloner', { programId: id, targetNode: selectedNode });
            }
            toast.success(`Instalación de ${id} completada exitosamente.`);
            addLog(`✅ Instalación de '${id}' finalizada correctamente.`);
            await handleVerify(id, selectedNode);
        } catch (error) {
            console.error(error);
            toast.error(`Fallo en la instalación: ${error}`);
            addLog(`❌ Error en instalación de '${id}': ${error}`);
        } finally {
            setInstalling(prev => ({ ...prev, [id]: false }));
        }
    };

    const handleAction = async (id: string, action: 'open' | 'close' | 'uninstall') => {
        const progStatus = status[id];
        if (!progStatus?.isInstalled || !progStatus.detectedPath) {
            toast.error("El programa no está instalado.");
            return;
        }

        setActionLoading(prev => ({ ...prev, [id]: true }));
        addLog(`Ejecutando acción '${action}' para '${id}' en ${selectedNode}...`);
        try {
            let command = '';
            if (action === 'open') command = 'bootstrapper_open_program';
            if (action === 'close') command = 'bootstrapper_close_program';
            if (action === 'uninstall') command = 'bootstrapper_uninstall_program';

            await invoke(command, { 
                programId: id, 
                targetNode: selectedNode, 
                detectedPath: progStatus.detectedPath 
            });
            toast.success(`Acción '${action}' ejecutada en ${selectedNode}.`);
            addLog(`✅ Acción '${action}' completada para '${id}'.`);
            if (action === 'uninstall') {
                await handleVerify(id, selectedNode);
            }
        } catch (error) {
            toast.error(`Error al realizar la acción: ${error}`);
            addLog(`❌ Fallo en '${action}' para '${id}': ${error}`);
            await handleVerify(id, selectedNode);
        } finally {
            setActionLoading(prev => ({ ...prev, [id]: false }));
        }
    };

    const saveProgramChanges = async (id: string) => {
        if (!config) return;
        const newPrograms = config.programs.map(p => {
            if (p.id === id) {
                return { ...p, ...editForm } as BootstrapperProgram;
            }
            return p;
        });
        
        const newConfig = { programs: newPrograms };
        try {
            await invoke('bootstrapper_save_config', { config: newConfig });
            setConfig(newConfig);
            setEditMode(null);
            toast.success("Rutas y configuración guardadas.");
            addLog(`Mapeo de rutas guardado para '${id}'.`);
            handleVerify(id);
        } catch (error) {
            toast.error(`Error al guardar: ${error}`);
        }
    };

    const isAntigravity = (id: string) => id.startsWith('antigravity');
    const isNetwork = (id: string) => ['tailscale', 'protonvpn', 'cloudflared'].includes(id);

    const filteredPrograms = config?.programs.filter(p => {
        if (selectedCategory === 'antigravity') return isAntigravity(p.id);
        if (selectedCategory === 'network') return isNetwork(p.id);
        if (selectedCategory === 'ecosystem') return !isAntigravity(p.id) && !isNetwork(p.id);
        return true;
    }) || [];

    if (!config) {
        return (
            <div className="flex-1 flex items-center justify-center h-full bg-slate-950 text-cyan-400">
                <div className="flex flex-col items-center gap-4">
                    <Loader2 className="w-12 h-12 animate-spin text-cyan-500" />
                    <p className="text-sm font-medium tracking-wide">Cargando Ecosistema Booster...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 overflow-auto p-6 md:p-8 h-full bg-slate-950 text-slate-100 font-sans selection:bg-cyan-500 selection:text-slate-950">
            <div className="max-w-6xl mx-auto space-y-8">
                
                {/* Header Principal */}
                <header className="relative p-8 rounded-2xl bg-gradient-to-r from-slate-900 via-slate-900/90 to-cyan-950/40 border border-slate-800/80 shadow-2xl backdrop-blur-xl overflow-hidden">
                    <div className="absolute top-0 right-0 p-12 bg-cyan-500/5 rounded-full blur-3xl pointer-events-none" />
                    
                    <div className="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
                        <div>
                            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-xs font-semibold tracking-wider uppercase mb-3">
                                <Sparkles className="w-3.5 h-3.5" /> Orquestador Universal Booster
                            </div>
                            <h1 className="text-3xl md:text-4xl font-extrabold text-white tracking-tight flex items-center gap-3">
                                Instalador & Booster
                            </h1>
                            <p className="text-slate-400 text-sm mt-2 max-w-2xl">
                                Gestión autónoma, mapeo inteligente, descarga directa y verificación en tiempo real para todas las herramientas de la Familia Google Antigravity y Civer Cloud.
                            </p>
                        </div>

                        {/* Selector de Nodo */}
                        <div className="flex flex-col gap-2 min-w-[280px] bg-slate-950/80 p-4 rounded-xl border border-slate-800 shadow-inner">
                            <label className="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-2">
                                <Server className="w-4 h-4 text-cyan-400" /> Destino de Despliegue:
                            </label>
                            <select 
                                className="bg-slate-900 text-slate-200 border border-slate-700/80 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-500/50 transition-all cursor-pointer"
                                value={selectedNode}
                                onChange={(e) => setSelectedNode(e.target.value)}
                            >
                                <option value="localhost">💻 Este Equipo (Localhost)</option>
                                <option value="100.96.218.12">💻 Laptop Civer Cloud (100.96.218.12)</option>
                                <option value="100.87.67.83">☁️ Gateway Principal (100.87.67.83)</option>
                            </select>
                        </div>
                    </div>

                    {/* Filtros de Categoría */}
                    <div className="mt-8 flex items-center gap-2 border-t border-slate-800/80 pt-6 overflow-x-auto">
                        <button 
                            onClick={() => setSelectedCategory('all')}
                            className={`px-4 py-2 rounded-lg text-xs font-semibold tracking-wide transition-all ${selectedCategory === 'all' ? 'bg-cyan-500 text-slate-950 shadow-lg shadow-cyan-500/20' : 'bg-slate-800/60 text-slate-400 hover:text-white hover:bg-slate-800'}`}
                        >
                            Todos los Programas ({config.programs.length})
                        </button>
                        <button 
                            onClick={() => setSelectedCategory('antigravity')}
                            className={`px-4 py-2 rounded-lg text-xs font-semibold tracking-wide transition-all ${selectedCategory === 'antigravity' ? 'bg-cyan-500 text-slate-950 shadow-lg shadow-cyan-500/20' : 'bg-slate-800/60 text-slate-400 hover:text-white hover:bg-slate-800'}`}
                        >
                            Familia Google Antigravity
                        </button>
                        <button 
                            onClick={() => setSelectedCategory('network')}
                            className={`px-4 py-2 rounded-lg text-xs font-semibold tracking-wide transition-all ${selectedCategory === 'network' ? 'bg-cyan-500 text-slate-950 shadow-lg shadow-cyan-500/20' : 'bg-slate-800/60 text-slate-400 hover:text-white hover:bg-slate-800'}`}
                        >
                            Red MESH & VPN
                        </button>
                        <button 
                            onClick={() => setSelectedCategory('ecosystem')}
                            className={`px-4 py-2 rounded-lg text-xs font-semibold tracking-wide transition-all ${selectedCategory === 'ecosystem' ? 'bg-cyan-500 text-slate-950 shadow-lg shadow-cyan-500/20' : 'bg-slate-800/60 text-slate-400 hover:text-white hover:bg-slate-800'}`}
                        >
                            Herramientas Ecosistema
                        </button>
                    </div>
                </header>

                {/* Grid de Programas */}
                <div className="grid gap-4">
                    {filteredPrograms.map(dep => {
                        const progStatus = status[dep.id];
                        const isInstalled = progStatus?.isInstalled;
                        const isLoading = actionLoading[dep.id] || installing[dep.id];

                        return (
                            <div 
                                key={dep.id} 
                                className="group relative bg-slate-900/60 hover:bg-slate-900/90 border border-slate-800/80 hover:border-cyan-500/40 rounded-xl p-6 shadow-xl transition-all duration-300 backdrop-blur-md"
                            >
                                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
                                    
                                    {/* Detalles del Programa */}
                                    <div className="space-y-2 flex-1">
                                        <div className="flex items-center gap-3">
                                            <h3 className="text-lg font-bold text-white group-hover:text-cyan-400 transition-colors">
                                                {dep.name}
                                            </h3>
                                            
                                            {/* Badge de Estado */}
                                            {isInstalled === true && (
                                                <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" /> Instalado
                                                </span>
                                            )}
                                            {isInstalled === false && (
                                                <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/30">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-rose-400" /> Faltante
                                                </span>
                                            )}
                                            {isInstalled === undefined && (
                                                <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
                                                    <Loader2 className="w-3 h-3 animate-spin" /> Escaneando...
                                                </span>
                                            )}
                                        </div>

                                        <p className="text-slate-400 text-sm leading-relaxed">
                                            {dep.description}
                                        </p>

                                        {isInstalled && progStatus?.detectedPath && (
                                            <div className="inline-flex items-center gap-2 mt-2 text-xs font-mono text-slate-400 bg-slate-950/80 px-3 py-1.5 rounded-lg border border-slate-800 break-all">
                                                <Folder className="w-3.5 h-3.5 text-cyan-400 flex-shrink-0" />
                                                <span>Ruta: {progStatus.detectedPath}</span>
                                            </div>
                                        )}
                                    </div>

                                    {/* Botones de Acción */}
                                    <div className="flex items-center gap-2 flex-wrap justify-end">
                                        <button 
                                            onClick={() => {
                                                setEditMode(editMode === dep.id ? null : dep.id);
                                                setEditForm(dep);
                                            }}
                                            className="p-2 rounded-lg bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-white transition-all border border-slate-700/50"
                                            title="Configurar Mapeo"
                                        >
                                            <SettingsIcon className="w-4 h-4" />
                                        </button>
                                        
                                        <button 
                                            onClick={() => handleVerify(dep.id)}
                                            disabled={isLoading}
                                            className="px-3 py-2 rounded-lg bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-white text-xs font-medium transition-all border border-slate-700/50 flex items-center gap-1.5"
                                        >
                                            <RefreshCw className="w-3.5 h-3.5" /> Verificar
                                        </button>

                                        {isInstalled ? (
                                            <>
                                                <button 
                                                    onClick={() => handleAction(dep.id, 'open')}
                                                    disabled={isLoading}
                                                    className="px-4 py-2 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-400 hover:text-emerald-300 border border-emerald-500/30 text-xs font-semibold transition-all flex items-center gap-2 shadow-lg shadow-emerald-500/5"
                                                >
                                                    {isLoading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5 fill-current" />}
                                                    Abrir
                                                </button>
                                                <button 
                                                    onClick={() => handleAction(dep.id, 'close')}
                                                    disabled={isLoading}
                                                    className="px-4 py-2 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 hover:text-amber-300 border border-amber-500/30 text-xs font-semibold transition-all flex items-center gap-2"
                                                >
                                                    <Square className="w-3.5 h-3.5 fill-current" /> Cerrar
                                                </button>
                                                <button 
                                                    onClick={() => handleAction(dep.id, 'uninstall')}
                                                    disabled={isLoading}
                                                    className="px-4 py-2 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 hover:text-rose-300 border border-rose-500/30 text-xs font-semibold transition-all flex items-center gap-2"
                                                >
                                                    <Trash2 className="w-3.5 h-3.5" /> Desinstalar
                                                </button>
                                            </>
                                        ) : (
                                            <button 
                                                onClick={() => handleInstall(dep.id)}
                                                disabled={isLoading}
                                                className="px-5 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-slate-950 font-bold text-xs shadow-lg shadow-cyan-500/20 transition-all flex items-center gap-2 cursor-pointer"
                                            >
                                                {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                                                Instalar Portable
                                            </button>
                                        )}
                                    </div>
                                </div>

                                {/* Formulario de Edición de Mapeo */}
                                {editMode === dep.id && (
                                    <div className="mt-4 pt-4 border-t border-slate-800 space-y-4 bg-slate-950/70 p-4 rounded-xl border border-slate-800">
                                        <h4 className="text-xs font-bold uppercase tracking-wider text-cyan-400 flex items-center gap-2">
                                            <SettingsIcon className="w-4 h-4" /> Configuración de Mapeo Dinámico
                                        </h4>
                                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div>
                                                <label className="text-xs text-slate-400 mb-1 block">Ruta Local (Ejecutable Target)</label>
                                                <input 
                                                    type="text" 
                                                    className="w-full bg-slate-900 border border-slate-700/80 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-cyan-500" 
                                                    value={editForm.local_path || ''}
                                                    onChange={e => setEditForm({...editForm, local_path: e.target.value})}
                                                    placeholder="C:\ProyectoCiverCloudUnificado\Herramientas\..."
                                                />
                                            </div>
                                            <div>
                                                <label className="text-xs text-slate-400 mb-1 block">URL Directa de Descarga</label>
                                                <input 
                                                    type="text" 
                                                    className="w-full bg-slate-900 border border-slate-700/80 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-cyan-500" 
                                                    value={editForm.download_url || ''}
                                                    onChange={e => setEditForm({...editForm, download_url: e.target.value})}
                                                    placeholder="https://..."
                                                />
                                            </div>
                                        </div>
                                        <div className="flex justify-end gap-2 pt-2">
                                            <button 
                                                onClick={() => setEditMode(null)}
                                                className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium"
                                            >
                                                Cancelar
                                            </button>
                                            <button 
                                                onClick={() => saveProgramChanges(dep.id)}
                                                className="px-4 py-1.5 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-slate-950 text-xs font-bold flex items-center gap-1.5 shadow-lg shadow-cyan-500/20"
                                            >
                                                <Save className="w-3.5 h-3.5" /> Guardar Mapeo
                                            </button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        );
                    })}
                </div>

                {/* Consola de Registros y Eventos en Vivo */}
                <div className="bg-slate-900/90 border border-slate-800 rounded-xl p-5 shadow-2xl backdrop-blur-xl">
                    <div className="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
                        <span className="text-xs font-mono font-bold uppercase tracking-wider text-slate-400 flex items-center gap-2">
                            <Terminal className="w-4 h-4 text-cyan-400" /> Consola de Eventos y Telemetría Booster
                        </span>
                        <span className="text-[10px] font-mono text-slate-500">Live Feedback</span>
                    </div>
                    <div className="font-mono text-xs space-y-1 max-h-40 overflow-y-auto bg-slate-950/90 p-3 rounded-lg border border-slate-800/80">
                        {logs.length === 0 ? (
                            <p className="text-slate-600 italic">Esperando eventos de verificación e instalación...</p>
                        ) : (
                            logs.map((log, idx) => (
                                <p key={idx} className={log.includes('✅') ? 'text-emerald-400' : log.includes('❌') ? 'text-rose-400' : 'text-slate-300'}>
                                    {log}
                                </p>
                            ))
                        )}
                    </div>
                </div>

            </div>
        </div>
    );
}
