class my_class
  def initialize
    @value = 10
  end
  
  def set_value(new_value)
    @value = new_value
  end
  
  def get_value
    puts "Value: #{@value}"
    @value
  end
end
