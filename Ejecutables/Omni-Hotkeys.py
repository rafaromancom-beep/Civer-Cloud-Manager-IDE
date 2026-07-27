import keyboard
import subprocess
import os
import sys
import threading
import pystray
from PIL import Image, ImageDraw
import tkinter as tk

IDE_DIR = r"C:\ProyectoCiverCloudUnificado\Desktop-y-Extensiones\Civer-Cloud-Manager-IDE"

def launch_dev():
    try:
        # Intenta usar Windows Terminal con diseño hermoso
        subprocess.Popen(['wt.exe', '-d', IDE_DIR, 'cmd', '/k', 'title Civer Cloud DEV ^& color 0b ^& npm run tauri dev'], cwd=IDE_DIR)
    except FileNotFoundError:
        # Fallback a un CMD personalizado si no hay wt.exe
        subprocess.Popen('start "Civer Cloud DEV" cmd /T:0B /k "npm run tauri dev"', shell=True, cwd=IDE_DIR)

def launch_build():
    script_path = os.path.join(IDE_DIR, "Ejecutables", "Build-UI.py")
    subprocess.Popen([sys.executable, script_path], cwd=IDE_DIR)

def launch_cloud_build():
    # Compilador Master con failover: GitHub Actions → Modal.com → Local
    master = os.path.join(IDE_DIR, "Ejecutables", "Cloud-Compiler-Master.py")
    legacy = os.path.join(IDE_DIR, "Ejecutables", "Build-Cloud-UI.py")
    script_path = master if os.path.exists(master) else legacy
    subprocess.Popen([sys.executable, script_path], cwd=IDE_DIR)

def launch_app():
    target_dir = os.path.join(IDE_DIR, "src-tauri", "target", "release")
    if os.path.exists(target_dir):
        for f in os.listdir(target_dir):
            if f.endswith(".exe") and not f.endswith("build.exe"):
                app_path = os.path.join(target_dir, f)
                subprocess.Popen(app_path)
                return

# --- GUI Window ---
def show_gui():
    # Only allow one instance of the GUI window
    if hasattr(show_gui, "window") and show_gui.window.winfo_exists():
        show_gui.window.lift()
        show_gui.window.focus_force()
        return

    root = tk.Tk()
    show_gui.window = root
    root.title("Civer Cloud - Panel de Control Omni")
    root.geometry("500x420")
    root.configure(bg="#1e1e2e")
    root.attributes("-topmost", True)
    
    # Center the window
    root.eval('tk::PlaceWindow . center')
    
    title = tk.Label(root, text="Civer Cloud Omni-Hotkeys", font=("Segoe UI", 16, "bold"), bg="#1e1e2e", fg="#cdd6f4")
    title.pack(pady=20)
    
    def create_btn(text, command, shortcut):
        frame = tk.Frame(root, bg="#1e1e2e")
        frame.pack(fill=tk.X, padx=40, pady=5)
        
        btn = tk.Button(frame, text=text, font=("Segoe UI", 12), bg="#89b4fa", fg="#11111b", 
                        activebackground="#b4befe", activeforeground="#11111b",
                        relief=tk.FLAT, command=command, width=25, cursor="hand2")
        btn.pack(side=tk.LEFT, padx=10)
        
        lbl = tk.Label(frame, text=shortcut, font=("Consolas", 10), bg="#1e1e2e", fg="#a6adc8")
        lbl.pack(side=tk.LEFT)

    create_btn("Lanzar Entorno Dev", launch_dev, "Ctrl + Shift + Alt + D")
    create_btn("Compilar Local (Build UI)", launch_build, "Ctrl + Shift + Alt + B")
    create_btn("Compilar en la Nube ☁️", launch_cloud_build, "Ctrl + Shift + Alt + C")
    create_btn("Abrir App Final", launch_app, "Ctrl + Shift + Alt + A")
    
    info = tk.Label(root, text="Este panel está corriendo en segundo plano.\nPuedes usar los atajos en cualquier momento.",
                    font=("Segoe UI", 10), bg="#1e1e2e", fg="#9399b2")
    info.pack(pady=20)
    
    root.mainloop()

# --- System Tray ---
def create_image():
    # Generate a simple icon 
    width = 64
    height = 64
    image = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    dc = ImageDraw.Draw(image)
    dc.ellipse((4, 4, 60, 60), fill="#89b4fa", outline="#cdd6f4", width=3)
    dc.text((22, 16), "C", fill="#1e1e2e", font=None)
    return image

def on_quit(icon, item):
    icon.stop()
    os._exit(0)

def setup_tray():
    image = create_image()
    menu = pystray.Menu(
        pystray.MenuItem("Abrir Panel de Control", lambda: threading.Thread(target=show_gui, daemon=True).start(), default=True),
        pystray.MenuItem("Cerrar Omni-Hotkeys", on_quit)
    )
    icon = pystray.Icon("CiverCloud", image, "Civer Cloud Omni-Hotkeys", menu)
    icon.run()

# --- Main ---
if __name__ == "__main__":
    keyboard.add_hotkey('ctrl+shift+alt+d', launch_dev)
    keyboard.add_hotkey('ctrl+shift+alt+b', launch_build)
    keyboard.add_hotkey('ctrl+shift+alt+c', launch_cloud_build)
    keyboard.add_hotkey('ctrl+shift+alt+a', launch_app)
    
    # Start the system tray icon
    setup_tray()
