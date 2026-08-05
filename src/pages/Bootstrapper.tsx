import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Download, CheckCircle, XCircle, Loader2, Save, FolderOpen, Settings as SettingsIcon } from 'lucide-react';
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
    const [status, setStatus] = useState<Record<string, 'installed' | 'missing' | 'unknown'>>({});
    const [editMode, setEditMode] = useState<string | null>(null);
    const [editForm, setEditForm] = useState<Partial<BootstrapperProgram>>({});

    const loadConfig = async () => {
        try {
            const data = await invoke<BootstrapperConfig>('bootstrapper_get_config');
            setConfig(data);
            
            // Auto-verify all on load
            data.programs.forEach(prog => handleVerify(prog.id));
        } catch (error) {
            console.error(error);
            toast.error("No se pudo cargar la configuración del Bootstrapper.");
        }
    };

    useEffect(() => {
        loadConfig();
    }, []);

    const handleVerify = async (id: string) => {
        try {
            const isInstalled = await invoke<boolean>('bootstrapper_verify_program', { programId: id });
            setStatus(prev => ({ ...prev, [id]: isInstalled ? 'installed' : 'missing' }));
        } catch (error) {
            console.error(error);
        }
    };

    const handleInstall = async (id: string) => {
        setInstalling(prev => ({ ...prev, [id]: true }));
        try {
            toast.info(`Iniciando instalación de ${id}...`);
            await invoke('bootstrapper_install_program', { programId: id });
            toast.success(`Instalación de ${id} completada.`);
            await handleVerify(id);
        } catch (error) {
            console.error(error);
            toast.error(`Fallo en la instalación: ${error}`);
        } finally {
            setInstalling(prev => ({ ...prev, [id]: false }));
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
            toast.success("Rutas actualizadas correctamente.");
            handleVerify(id);
        } catch (error) {
            toast.error(`Error guardando: ${error}`);
        }
    };

    if (!config) return <div className="p-8"><Loader2 className="w-8 h-8 animate-spin" /></div>;

    return (
        <div className="flex-1 overflow-auto p-8 relative h-full bg-[#faf8f5] dark:bg-base-300 transition-colors">
            <div className="max-w-4xl mx-auto space-y-6">
                <header className="mb-8">
                    <h1 className="text-3xl font-bold text-base-content flex items-center gap-3">
                        <Download className="w-8 h-8 text-primary" />
                        Instalador & Booster
                    </h1>
                    <p className="text-base-content/70 mt-2">
                        Configuración inicial, mapeo y verificación de programas de la Familia Antigravity y Civer Cloud.
                    </p>
                </header>

                <div className="grid gap-4">
                    {config.programs.map(dep => (
                        <div key={dep.id} className="bg-base-100 dark:bg-base-200 p-6 rounded-xl shadow-sm border border-base-200 dark:border-base-100 flex flex-col gap-4">
                            <div className="flex items-center justify-between">
                                <div>
                                    <h3 className="text-lg font-semibold text-base-content">{dep.name}</h3>
                                    <p className="text-base-content/70 text-sm mt-1">{dep.description}</p>
                                    <div className="mt-2 flex items-center gap-2 text-sm font-medium">
                                        Estado: 
                                        {status[dep.id] === 'installed' && <span className="text-success flex items-center gap-1"><CheckCircle className="w-4 h-4"/> Instalado</span>}
                                        {status[dep.id] === 'missing' && <span className="text-error flex items-center gap-1"><XCircle className="w-4 h-4"/> Faltante</span>}
                                        {(!status[dep.id] || status[dep.id] === 'unknown') && <span className="text-base-content/50">Escaneando...</span>}
                                    </div>
                                </div>
                                <div className="flex items-center gap-3">
                                    <button 
                                        onClick={() => {
                                            setEditMode(editMode === dep.id ? null : dep.id);
                                            setEditForm(dep);
                                        }}
                                        className="btn btn-ghost btn-sm btn-circle"
                                        title="Configurar Rutas"
                                    >
                                        <SettingsIcon className="w-4 h-4" />
                                    </button>
                                    <button 
                                        onClick={() => handleVerify(dep.id)}
                                        disabled={installing[dep.id]}
                                        className="btn btn-outline btn-sm"
                                    >
                                        Verificar
                                    </button>
                                    <button 
                                        onClick={() => handleInstall(dep.id)}
                                        disabled={installing[dep.id]}
                                        className="btn btn-primary btn-sm flex items-center gap-2"
                                    >
                                        {installing[dep.id] ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                                        Instalar
                                    </button>
                                </div>
                            </div>

                            {editMode === dep.id && (
                                <div className="bg-base-200/50 p-4 rounded-lg mt-2 space-y-3 border border-base-300">
                                    <h4 className="text-sm font-semibold mb-2">Mapeo de Rutas</h4>
                                    <div>
                                        <label className="text-xs font-medium text-base-content/70">Ruta Local (Ejecutable)</label>
                                        <input 
                                            type="text" 
                                            className="input input-sm input-bordered w-full mt-1" 
                                            value={editForm.local_path || ''}
                                            onChange={e => setEditForm({...editForm, local_path: e.target.value})}
                                            placeholder="C:\Ruta\Al\Programa.exe"
                                        />
                                    </div>
                                    <div>
                                        <label className="text-xs font-medium text-base-content/70">URL de Descarga</label>
                                        <input 
                                            type="text" 
                                            className="input input-sm input-bordered w-full mt-1" 
                                            value={editForm.download_url || ''}
                                            onChange={e => setEditForm({...editForm, download_url: e.target.value})}
                                            placeholder="https://..."
                                        />
                                    </div>
                                    <div className="flex justify-end pt-2">
                                        <button 
                                            onClick={() => saveProgramChanges(dep.id)}
                                            className="btn btn-sm btn-secondary flex items-center gap-2"
                                        >
                                            <Save className="w-4 h-4"/> Guardar Mapeo
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
