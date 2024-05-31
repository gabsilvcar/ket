import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# Load the CSV file using Pandas
df = pd.read_csv('benchmarks-random.csv')
palette = 'magma'

# Define a dictionary to map English names to Brazilian Portuguese
name_mapping = {
    'original-t-gates': 'original',
    'quizx-clifford-simplified-t-gates': 'simplificação de clifford ket',
    'pyzx-clifford-simplified-t-gates': 'simplificação de clifford pyzx',
    # 'pyzx-teleport-reduced-t-gates': 'simplificação de teleporte pyzx',
    'pyzx-full-reduction-t-gates': 'redução completa pyzx'
}

# Reshape the DataFrame to long format and apply the name mapping
df_long = df.melt(id_vars='p(t)', value_vars=list(name_mapping.keys()), var_name='Method', value_name='Number of Gates')
df_long['Method'] = df_long['Method'].map(name_mapping)

# Create a scatter plot with different colors and labels for each method
sns.lineplot(x='p(t)', y='Number of Gates', hue='Method', data=df_long, palette=palette)
sns.scatterplot(x='p(t)', y='Number of Gates', hue='Method', data=df_long, palette=palette, marker='s', legend=False)

# plt.yscale('log')
# plt.grid(True, linestyle='-', linewidth=0.5, color='grey', alpha=0.5)

# Customizing legend
plt.legend(title='Método de Redução', frameon=False, fontsize='small')


plt.xlabel('Proporção de portas T')
plt.ylabel('Quantidade de portas T')

plt.savefig("/tmp/tgates.svg", transparent=True)
