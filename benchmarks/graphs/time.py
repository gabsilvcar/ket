import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# Load the CSV file using Pandas
df = pd.read_csv('benchmarks-random.csv')
palette = 'magma'  # Choosing a different palette for better visibility

# Define a dictionary to map English method names to Brazilian Portuguese
name_mapping = {
    'quizx-clifford-simplified-time': 'simplificação de clifford ket',
    'pyzx-clifford-simplified-time': 'simplificação de clifford pyzx',
    # 'pyzx-teleport-reduced-time': 'tempo reduzido por teleportação PyZX',
    'pyzx-full-reduction-time': 'redução completa pyzx'
}

# Assuming your CSV has columns like 'Number of Gates' and the times for each method
# Melt the DataFrame to long format for plotting
df_long = df.melt(id_vars='p(t)', value_vars=list(name_mapping.keys()), var_name='Method', value_name='Time')
df_long['Method'] = df_long['Method'].map(name_mapping)
df_long['Time'] *= 1000

# TODO olhar aqui
# df_long = df_long[df_long['original-gates'].between(0, 500)]


# Create a scatter plot
sns.scatterplot(x='p(t)', y='Time', hue='Method', data=df_long, palette=palette)
plt.yscale('log')
plt.legend(title='Método de Redução', frameon=False, fontsize='small')

plt.xlabel('Proporção de portas T')
plt.ylabel('Tempo (milisegundos)')
# plt.title('Relação entre quantidade de portas e tempo de redução')

plt.savefig("/tmp/time.svg", transparent=True)
