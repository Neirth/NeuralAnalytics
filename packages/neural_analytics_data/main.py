"""
Sistema de Captura EEG con BrainBit
Punto de entrada principal para la aplicación.

Este script inicializa el controlador principal que gestiona todo el flujo
de captura de datos EEG, desde la configuración del dispositivo hasta
el almacenamiento de los datos capturados.

Ejemplos de uso:
    # Captura con escenario rojo (predeterminado)
    python main.py 
    
    # Captura con escenario verde
    python main.py --type green
    
    # Especificar MAC personalizada
    python main.py --mac C8:8F:B6:6D:E1:E2
"""

import sys
import traceback
import argparse

from controllers.main_controller import NeuralCaptureController

def main():
    """
    Función principal para iniciar la aplicación de captura EEG.
    Procesa los argumentos de línea de comandos y ejecuta el controlador.
    """
    print("Iniciando Sistema de Captura EEG con BrainBit...\n")
    
    # Configurar parser para argumentos de línea de comandos
    parser = argparse.ArgumentParser(
        prog="Neural Capturer",
        description="Sistema de Captura EEG con BrainBit"
    )
    
    # Argumentos disponibles
    parser.add_argument('--type', choices=['red', 'green', 'trash'], default='red',
                      help='Tipo de escenario (red/green)')
    parser.add_argument('--mac',
                      help='Dirección MAC del BrainBit (formato: "A0:B1:C2:D3:E4:F5")')
    
    # Parsear argumentos
    args = parser.parse_args()
    
    try:
        # Crear controlador con los parámetros especificados
        controller = NeuralCaptureController(
            scenario_type=args.type,
            mac_address=args.mac
        )
        
        # Ejecutar el flujo principal de captura
        print(f"Iniciando captura con escenario: {args.type.upper()}")
        if args.mac:
            print(f"Usando dirección MAC personalizada: {args.mac}")
        
        success = controller.run()
        
        # Mostrar resultado
        if success:
            print("\n[✓] Captura completada con éxito.")
            print("    Los archivos se han guardado en el directorio data/")
            return 0
        else:
            print("\n[!] La captura se canceló o no se completó correctamente.")
            return 1
            
    except KeyboardInterrupt:
        print("\n\n[!] Operación cancelada por el usuario (Ctrl+C).")
        return 1
    except Exception as e:
        print(f"\n[!] Error inesperado: {str(e)}")
        print("\nDetalles del error:")
        traceback.print_exc()
        return 2

if __name__ == "__main__":
    sys.exit(main())